package com.aegisec.backend.repository;

import com.aegisec.backend.model.Event;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.stereotype.Repository;

import java.util.List;

@Repository
public interface EventRepository extends JpaRepository<Event, Long> {
    List<Event> findTop50ByOrderByTimestampDesc();

    List<Event> findByAgentIdOrderByTimestampDesc(String agentId);
}
