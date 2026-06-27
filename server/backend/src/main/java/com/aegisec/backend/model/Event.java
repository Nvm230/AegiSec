package com.aegisec.backend.model;

import jakarta.persistence.*;
import lombok.Data;
import lombok.NoArgsConstructor;
import org.hibernate.annotations.CreationTimestamp;

import java.time.Instant;

@Entity
@Table(name = "events")
@Data
@NoArgsConstructor
public class Event {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(nullable = false)
    private String agentId;

    @Column(nullable = false)
    private String hostname;

    private String osInfo;

    @CreationTimestamp
    @Column(nullable = false, updatable = false)
    private Instant timestamp;

    @Column(nullable = false)
    private String eventType; // PROCESS_SPAWN, NET_CONNECT, AUTH_FAIL

    private String processName;
    private String cmdline;
    private Double riskScore;
    private String actionTaken; // KILL_PROCESS, BLOCK_IP, NONE

    @Enumerated(EnumType.STRING)
    private Severity severity = Severity.INFO;

    public enum Severity {
        INFO, SUSPICIOUS, BLOCKED
    }
}
