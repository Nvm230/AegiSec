package com.aegisec.backend.controller;

import com.aegisec.backend.model.Event;
import com.aegisec.backend.repository.EventRepository;
import com.aegisec.backend.service.TelemetryService;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

import java.util.List;

@RestController
@RequestMapping("/api/v1")
@RequiredArgsConstructor
public class TelemetryController {

    private final TelemetryService telemetryService;
    private final EventRepository eventRepository;

    /**
     * Agent -> Backend: ingest a new security event.
     * Authenticated via mTLS (client cert at TLS layer).
     */
    @PostMapping("/events")
    public ResponseEntity<Event> ingestEvent(@RequestBody @Valid Event event) {
        Event saved = telemetryService.ingestEvent(event);
        return ResponseEntity.ok(saved);
    }

    /**
     * Dashboard: get recent events (latest 50).
     */
    @GetMapping("/events")
    public ResponseEntity<List<Event>> getRecentEvents() {
        return ResponseEntity.ok(eventRepository.findTop50ByOrderByTimestampDesc());
    }

    /**
     * Dashboard: get events for a specific agent.
     */
    @GetMapping("/events/{agentId}")
    public ResponseEntity<List<Event>> getEventsByAgent(@PathVariable String agentId) {
        return ResponseEntity.ok(eventRepository.findByAgentIdOrderByTimestampDesc(agentId));
    }
}
