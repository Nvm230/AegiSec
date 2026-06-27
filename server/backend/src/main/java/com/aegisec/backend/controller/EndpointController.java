package com.aegisec.backend.controller;

import com.aegisec.backend.model.Endpoint;
import com.aegisec.backend.repository.EndpointRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

import java.util.List;

@RestController
@RequestMapping("/api/v1/endpoints")
@RequiredArgsConstructor
public class EndpointController {

    private final EndpointRepository endpointRepository;

    /** Dashboard: list all monitored endpoints. */
    @GetMapping
    public ResponseEntity<List<Endpoint>> listEndpoints() {
        return ResponseEntity.ok(endpointRepository.findAll());
    }

    /** Dashboard: get specific endpoint. */
    @GetMapping("/{id}")
    public ResponseEntity<Endpoint> getEndpoint(@PathVariable Long id) {
        return endpointRepository.findById(id)
                .map(ResponseEntity::ok)
                .orElse(ResponseEntity.notFound().build());
    }

    /** Dashboard: manual command — update endpoint state (e.g. un-block). */
    @PutMapping("/{id}/state")
    public ResponseEntity<Endpoint> updateState(@PathVariable Long id,
            @RequestParam Endpoint.RiskState state) {
        return endpointRepository.findById(id)
                .map(endpoint -> {
                    endpoint.setRiskState(state);
                    endpoint.setCurrentRiskScore(0.0);
                    return ResponseEntity.ok(endpointRepository.save(endpoint));
                })
                .orElse(ResponseEntity.notFound().build());
    }
}
