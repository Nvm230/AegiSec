package com.aegisec.backend.model;

import jakarta.persistence.*;
import lombok.Data;
import lombok.NoArgsConstructor;
import org.hibernate.annotations.UpdateTimestamp;

import java.time.Instant;

@Entity
@Table(name = "endpoints")
@Data
@NoArgsConstructor
public class Endpoint {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(nullable = false, unique = true)
    private String agentId;

    private String hostname;
    private String ip;
    private String osInfo;

    @Enumerated(EnumType.STRING)
    private Status status = Status.ONLINE;

    @UpdateTimestamp
    private Instant lastPing;

    private Double currentRiskScore = 0.0;

    @Enumerated(EnumType.STRING)
    private RiskState riskState = RiskState.CLEAN;

    public enum Status {
        ONLINE, OFFLINE
    }

    public enum RiskState {
        CLEAN, SUSPICIOUS, BLOCKED
    }
}
