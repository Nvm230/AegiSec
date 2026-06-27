package com.aegisec.backend.service;

import com.aegisec.backend.model.Endpoint;
import com.aegisec.backend.model.Event;
import com.aegisec.backend.repository.EndpointRepository;
import com.aegisec.backend.repository.EventRepository;
import com.aegisec.backend.websocket.AlertBroadcaster;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.time.Instant;

@Service
@RequiredArgsConstructor
@Slf4j
public class TelemetryService {

    private final EventRepository eventRepository;
    private final EndpointRepository endpointRepository;
    private final AlertBroadcaster alertBroadcaster;

    public Event ingestEvent(Event event) {
        // 1. Upsert endpoint
        Endpoint endpoint = endpointRepository.findByAgentId(event.getAgentId())
                .orElse(new Endpoint());
        endpoint.setAgentId(event.getAgentId());
        endpoint.setHostname(event.getHostname());
        endpoint.setOsInfo(event.getOsInfo());
        endpoint.setStatus(Endpoint.Status.ONLINE);
        endpoint.setLastPing(Instant.now());

        if (event.getRiskScore() != null) {
            endpoint.setCurrentRiskScore(event.getRiskScore());
        }

        // 2. Update endpoint risk state
        if (event.getRiskScore() != null) {
            if (event.getRiskScore() >= 80) {
                endpoint.setRiskState(Endpoint.RiskState.BLOCKED);
                event.setSeverity(Event.Severity.BLOCKED);
            } else if (event.getRiskScore() >= 40) {
                endpoint.setRiskState(Endpoint.RiskState.SUSPICIOUS);
                event.setSeverity(Event.Severity.SUSPICIOUS);
            } else {
                endpoint.setRiskState(Endpoint.RiskState.CLEAN);
                event.setSeverity(Event.Severity.INFO);
            }
        }

        endpointRepository.save(endpoint);

        // 3. Save event
        Event saved = eventRepository.save(event);
        log.info("[TELEMETRY] {} | {} | score={} | action={}",
                saved.getEventType(), saved.getHostname(),
                saved.getRiskScore(), saved.getActionTaken());

        // 4. Broadcast alert via WebSocket if suspicious or blocked
        if (saved.getSeverity() != Event.Severity.INFO) {
            alertBroadcaster.broadcast(saved);
        }

        return saved;
    }
}
