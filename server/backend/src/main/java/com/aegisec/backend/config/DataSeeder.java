package com.aegisec.backend.config;

import com.aegisec.backend.model.AppUser;
import com.aegisec.backend.repository.UserRepository;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.ApplicationRunner;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.security.crypto.password.PasswordEncoder;

@Configuration
@RequiredArgsConstructor
@Slf4j
public class DataSeeder {

    @Bean
    public ApplicationRunner seedDefaultUser(UserRepository userRepository, PasswordEncoder passwordEncoder) {
        return args -> {
            if (userRepository.findByUsername("admin").isEmpty()) {
                AppUser admin = AppUser.builder()
                        .username("admin")
                        .passwordHash(passwordEncoder.encode("aegisec2025"))
                        .role(AppUser.Role.ADMIN)
                        .build();
                userRepository.save(admin);
                log.info("[SEED] Created default admin user: admin / aegisec2025");
            }
        };
    }
}
