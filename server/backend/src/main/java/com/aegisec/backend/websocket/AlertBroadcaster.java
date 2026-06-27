package com.aegisec.backend.websocket;

import com.aegisec.backend.model.Event;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.messaging.simp.SimpMessagingTemplate;
import org.springframework.stereotype.Component;

@Component
@RequiredArgsConstructor
@Slf4j
public class AlertBroadcaster {

    private final SimpMessagingTemplate messagingTemplate;

    public void broadcast(Event event) {
        log.debug("[WS] Broadcasting alert: {} on {}", event.getEventType(), event.getHostname());
        messagingTemplate.convertAndSend("/topic/alerts", event);
    }
}
