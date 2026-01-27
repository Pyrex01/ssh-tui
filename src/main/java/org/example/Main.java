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
        SshServer sshd = SshServer.setUpDefaultServer();
        sshd.setPort(2222);

        CustomShellFactory customShellFactory = new CustomShellFactory();
        sshd.setKeyPairProvider(
            new SimpleGeneratorHostKeyProvider()
        );
        sshd.setPublickeyAuthenticator(((username, key, session) -> true));
        sshd.setShellFactory(customShellFactory);
        sshd.start();
        System.out.println("🚀 Java SSH server running on port 2222");

        while (sshd.isOpen()) ;
    }
}
