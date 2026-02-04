package org.example;

import java.io.IOException;
import org.apache.sshd.server.channel.ChannelSession;
import org.apache.sshd.server.command.Command;
import org.apache.sshd.server.shell.ShellFactory;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class CustomShellFactory implements ShellFactory {

    private static final Logger logger = LoggerFactory.getLogger(CustomShellFactory.class);

    @Override
    public Command createShell(ChannelSession channel) throws IOException {
        logger.info("═══════════════════════════════════════════════════════════");
        logger.info("CustomShellFactory.createShell() CALLED!");
        logger.info("Channel: {}", channel);
        if (channel != null) {
            logger.info("Session: {}", channel.getSession());
        }
        logger.info("═══════════════════════════════════════════════════════════");
        try {
            CustomCommand command = new CustomCommand(channel);
            logger.info("CustomCommand created successfully: {}", command);
            return command;
        } catch (Exception e) {
            logger.error("ERROR creating CustomCommand!", e);
            throw e;
        }
    }
}
