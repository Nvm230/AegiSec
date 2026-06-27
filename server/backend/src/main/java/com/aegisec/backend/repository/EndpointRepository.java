package com.aegisec.backend.repository;

import com.aegisec.backend.model.Endpoint;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.stereotype.Repository;

import java.util.Optional;

@Repository
public interface EndpointRepository extends JpaRepository<Endpoint, Long> {
    Optional<Endpoint> findByAgentId(String agentId);
}
