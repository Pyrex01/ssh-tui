package org.example;

import java.io.IOException;
import org.apache.sshd.server.channel.ChannelSession;
import org.apache.sshd.server.command.Command;
import org.apache.sshd.server.shell.ShellFactory;

public class CustomShellFactory implements ShellFactory {

    @Override
    public Command createShell(ChannelSession channel) throws IOException {
        return new CustomCommand(channel);
    }
}
