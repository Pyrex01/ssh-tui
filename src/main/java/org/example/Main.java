package org.example;

import org.apache.sshd.server.SshServer;
import org.apache.sshd.server.keyprovider.SimpleGeneratorHostKeyProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

//TIP To <b>Run</b> code, press <shortcut actionId="Run"/> or
// click the <icon src="AllIcons.Actions.Execute"/> icon in the gutter.
public class Main {
    private static final Logger logger = LoggerFactory.getLogger(Main.class);

    public static void main(String[] args) throws Exception {
        logger.info("Starting SSH server...");
        SshServer sshd = SshServer.setUpDefaultServer();
        sshd.setPort(2222);
        logger.info("SSH server port set to 2222");

        CustomShellFactory customShellFactory = new CustomShellFactory();
        logger.info("CustomShellFactory created: {}", customShellFactory);
        
        sshd.setKeyPairProvider(
            new SimpleGeneratorHostKeyProvider()
        );
        logger.info("Key pair provider set");
        
        sshd.setPublickeyAuthenticator(((username, key, session) -> {
            logger.info("Authentication attempt for user: {}", username);
            return true;
        }));
        logger.info("Public key authenticator set (accepting all)");
        
        sshd.setShellFactory(customShellFactory);
        logger.info("Shell factory set to CustomShellFactory");
        
        sshd.start();
        logger.info("SSH server started successfully");
        System.out.println("🚀 Java SSH server running on port 2222");
        logger.info("Waiting for connections...");

        while (sshd.isOpen()) {
            Thread.sleep(1000);
        }
        logger.info("SSH server closed");
    }
}
