//! Process sensor — uses the Linux Netlink proc connector to receive
//! real-time FORK and EXEC events directly from the kernel.
//! Requires CAP_NET_ADMIN (i.e. root). Falls back to procfs polling if
//! the Netlink socket cannot be bound.

use std::collections::HashSet;
use std::mem;
use std::sync::mpsc::Sender;
use std::time::Duration;

use libc::{
    bind, c_void, close, getsockname, recv, send,
    sockaddr_nl, socket, AF_NETLINK, SOCK_DGRAM,
};
use tracing::{debug, error, info, warn};

use super::SensorEvent;

// Netlink constants from <linux/connector.h> and <linux/cn_proc.h>
const NETLINK_CONNECTOR: i32 = 11;
const NLMSG_DONE: u16 = 3;
const NLMSG_MIN_TYPE: u16 = 16;

const CN_IDX_PROC: u32 = 1;
const CN_VAL_PROC: u32 = 1;

const PROC_CN_MCAST_LISTEN: u32 = 1;

const PROC_EVENT_FORK: u32 = 0x00000002;
const PROC_EVENT_EXEC: u32 = 0x00000004;
const PROC_EVENT_EXIT: u32 = 0x80000000;

#[repr(C)]
struct NlMsgHdr {
    nlmsg_len: u32,
    nlmsg_type: u16,
    nlmsg_flags: u16,
    nlmsg_seq: u32,
    nlmsg_pid: u32,
}

#[repr(C)]
struct CbId {
    idx: u32,
    val: u32,
}

#[repr(C)]
struct CnMsg {
    id: CbId,
    seq: u32,
    ack: u32,
    len: u16,
    flags: u16,
}

#[repr(C)]
struct ProcEventHdr {
    what: u32,
    cpu: u32,
    timestamp_ns: u64,
}

#[repr(C)]
struct ForkEvent {
    parent_pid: u32,
    parent_tgid: u32,
    child_pid: u32,
    child_tgid: u32,
}

#[repr(C)]
struct ExecEvent {
    process_pid: u32,
    process_tgid: u32,
}

#[repr(C)]
struct ExitEvent {
    process_pid: u32,
    process_tgid: u32,
    exit_code: u32,
    exit_signal: u32,
}

/// Spawn process sensor. Tries Netlink first, falls back to procfs polling.
pub fn run(tx: Sender<SensorEvent>) {
    if try_netlink(tx.clone()) {
        return;
    }
    warn!("[PROCESS SENSOR] Netlink failed or insufficient privileges, using procfs polling.");
    run_procfs_poll(tx);
}

/// Attempt to set up Netlink proc connector.
/// Returns true if Netlink was successfully bound (runs loop until error).
fn try_netlink(tx: Sender<SensorEvent>) -> bool {
    let fd = unsafe { socket(AF_NETLINK, SOCK_DGRAM, NETLINK_CONNECTOR) };
    if fd < 0 {
        warn!("[PROCESS SENSOR] Cannot open Netlink socket (not root?)");
        return false;
    }

    let mut nl_addr: sockaddr_nl = unsafe { std::mem::zeroed() };
    nl_addr.nl_family = AF_NETLINK as u16;
    nl_addr.nl_pid = 0;
    nl_addr.nl_groups = CN_IDX_PROC;

    let bind_result = unsafe {
        bind(
            fd,
            &nl_addr as *const sockaddr_nl as *const _,
            mem::size_of::<sockaddr_nl>() as u32,
        )
    };

    if bind_result < 0 {
        unsafe { close(fd) };
        warn!("[PROCESS SENSOR] Netlink bind failed (errno={})", unsafe { *libc::__errno_location() });
        return false;
    }

    // Send MCAST_LISTEN to enable proc events
    if !send_mcast_listen(fd) {
        unsafe { close(fd) };
        return false;
    }

    info!("[PROCESS SENSOR] ✅ Netlink proc connector active — real-time FORK/EXEC monitoring.");

    // Receive loop
    let mut buf = vec![0u8; 4096];
    loop {
        let len = unsafe { recv(fd, buf.as_mut_ptr() as *mut c_void, buf.len(), 0) };
        if len <= 0 {
            error!("[PROCESS SENSOR] Netlink recv failed, restarting...");
            break;
        }
        parse_netlink_message(&buf[..len as usize], &tx);
    }

    unsafe { close(fd) };
    true
}

/// Send PROC_CN_MCAST_LISTEN command to kernel
fn send_mcast_listen(fd: libc::c_int) -> bool {
    // total: NlMsgHdr + CnMsg + u32 (op)
    let cn_msg_size = mem::size_of::<NlMsgHdr>() + mem::size_of::<CnMsg>() + mem::size_of::<u32>();

    let mut buf = vec![0u8; cn_msg_size];
    let ptr = buf.as_mut_ptr();

    unsafe {
        let nl = &mut *(ptr as *mut NlMsgHdr);
        nl.nlmsg_len = cn_msg_size as u32;
        nl.nlmsg_type = NLMSG_MIN_TYPE;
        nl.nlmsg_flags = 0;
        nl.nlmsg_seq = 0;
        nl.nlmsg_pid = libc::getpid() as u32;

        let cn = &mut *((ptr.add(mem::size_of::<NlMsgHdr>())) as *mut CnMsg);
        cn.id.idx = CN_IDX_PROC;
        cn.id.val = CN_VAL_PROC;
        cn.seq = 0;
        cn.ack = 0;
        cn.len = mem::size_of::<u32>() as u16;
        cn.flags = 0;

        let op = &mut *((ptr.add(mem::size_of::<NlMsgHdr>() + mem::size_of::<CnMsg>())) as *mut u32);
        *op = PROC_CN_MCAST_LISTEN;

        let mut nl_dest: sockaddr_nl = std::mem::zeroed();
        nl_dest.nl_family = AF_NETLINK as u16;
        nl_dest.nl_pid = 0; // kernel
        nl_dest.nl_groups = 0;

        let ret = libc::sendto(
            fd,
            buf.as_ptr() as *const c_void,
            cn_msg_size,
            0,
            &nl_dest as *const sockaddr_nl as *const _,
            mem::size_of::<sockaddr_nl>() as u32,
        );
        ret > 0
    }
}

fn parse_netlink_message(buf: &[u8], tx: &Sender<SensorEvent>) {
    let nl_hdr_size = mem::size_of::<NlMsgHdr>();
    let cn_msg_size = mem::size_of::<CnMsg>();
    let ev_hdr_size = mem::size_of::<ProcEventHdr>();

    if buf.len() < nl_hdr_size + cn_msg_size + ev_hdr_size {
        return;
    }

    let offset = nl_hdr_size + cn_msg_size;
    let ev_hdr = unsafe { &*(buf[offset..].as_ptr() as *const ProcEventHdr) };

    let data_offset = offset + ev_hdr_size;

    match ev_hdr.what {
        PROC_EVENT_FORK => {
            if buf.len() < data_offset + mem::size_of::<ForkEvent>() {
                return;
            }
            let ev = unsafe { &*(buf[data_offset..].as_ptr() as *const ForkEvent) };
            let child_pid = ev.child_pid;
            let parent_pid = ev.parent_pid;

            // Read child info from procfs (with retry for exec race)
            std::thread::sleep(Duration::from_millis(5));
            let (uid, comm, cmdline) = read_proc_info(child_pid);
            debug!("[NETLINK] FORK: {} -> child PID={}", parent_pid, child_pid);

            let _ = tx.send(SensorEvent::ProcessSpawn {
                pid: child_pid,
                ppid: parent_pid,
                uid,
                comm,
                cmdline,
            });
        }
        PROC_EVENT_EXEC => {
            if buf.len() < data_offset + mem::size_of::<ExecEvent>() {
                return;
            }
            let ev = unsafe { &*(buf[data_offset..].as_ptr() as *const ExecEvent) };
            let pid = ev.process_pid;

            std::thread::sleep(Duration::from_millis(5));
            let (uid, comm, cmdline) = read_proc_info(pid);
            let ppid = read_ppid(pid);
            debug!("[NETLINK] EXEC: PID={} comm={} cmd={}", pid, comm, cmdline);

            let _ = tx.send(SensorEvent::ProcessSpawn {
                pid,
                ppid,
                uid,
                comm,
                cmdline,
            });
        }
        PROC_EVENT_EXIT => {
            if buf.len() < data_offset + mem::size_of::<ExitEvent>() {
                return;
            }
            let ev = unsafe { &*(buf[data_offset..].as_ptr() as *const ExitEvent) };
            let _ = tx.send(SensorEvent::ProcessExit { pid: ev.process_pid });
        }
        _ => {}
    }
}

fn read_proc_info(pid: u32) -> (u32, String, String) {
    let uid = std::fs::read_to_string(format!("/proc/{}/status", pid))
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|u| u.parse::<u32>().ok())
        })
        .unwrap_or(0);

    let comm = std::fs::read_to_string(format!("/proc/{}/comm", pid))
        .unwrap_or_default()
        .trim()
        .to_string();

    let cmdline = std::fs::read(format!("/proc/{}/cmdline", pid))
        .unwrap_or_default()
        .iter()
        .map(|&b| if b == 0 { b' ' } else { b })
        .collect::<Vec<u8>>();
    let cmdline = String::from_utf8_lossy(&cmdline).trim().to_string();

    (uid, comm, cmdline)
}

fn read_ppid(pid: u32) -> u32 {
    procfs::process::Process::new(pid as i32)
        .ok()
        .and_then(|p| p.stat().ok())
        .map(|s| s.ppid as u32)
        .unwrap_or(0)
}

/// Fallback: poll /proc for new processes every 500ms.
fn run_procfs_poll(tx: Sender<SensorEvent>) {
    info!("[PROCESS SENSOR] Polling /proc every 500ms...");
    let mut seen: HashSet<u32> = HashSet::new();

    loop {
        if let Ok(procs) = procfs::process::all_processes() {
            for proc in procs.flatten() {
                let pid = proc.pid as u32;
                if seen.contains(&pid) {
                    continue;
                }
                seen.insert(pid);

                let Ok(stat) = proc.stat() else { continue };
                let ppid = stat.ppid as u32;
                let uid = proc.status().ok().map(|s| s.ruid as u32).unwrap_or(0);
                let comm = stat.comm.clone();
                let cmdline = proc
                    .cmdline()
                    .ok()
                    .map(|v| v.join(" "))
                    .unwrap_or_default();

                let _ = tx.send(SensorEvent::ProcessSpawn { pid, ppid, uid, comm, cmdline });
            }
            // Prune dead PIDs
            seen.retain(|&pid| std::path::Path::new(&format!("/proc/{}", pid)).exists());
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}
