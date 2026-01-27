package org.example;

import java.io.InputStream;
import java.io.OutputStream;
import org.apache.sshd.server.Environment;
import org.apache.sshd.server.ExitCallback;
import org.apache.sshd.server.channel.ChannelSession;
import org.apache.sshd.server.command.Command;

public class CustomCommand implements Command, Runnable {

    private InputStream in;
    private OutputStream out;
    private OutputStream err;
    private ExitCallback exitCallback;
    private final ChannelSession channelSession;
    private Thread worker;
    private volatile boolean running = true;

    private Environment environment;

    public CustomCommand(ChannelSession channel) {
        this.channelSession = channel;
    }

    // ===== Stream setters =====

    @Override
    public void setInputStream(InputStream in) {
        this.in = in;
    }

    @Override
    public void setOutputStream(OutputStream out) {
        this.out = out;
    }

    @Override
    public void setErrorStream(OutputStream err) {
        this.err = err;
    }

    @Override
    public void setExitCallback(ExitCallback callback) {
        this.exitCallback = callback;
    }

    // ===== Lifecycle =====

    @Override
    public void start(ChannelSession channel, Environment env) {
        this.environment = env;
        worker = new Thread(this, "ssh-lanterna-session");
        worker.start();
    }

    @Override
    public void destroy(ChannelSession channel) {
        running = false;
        if (worker != null) {
            worker.interrupt();
        }
    }

    // ===== Main logic =====

    @Override
    public void run() {
        try {
            HelloJexer app = new HelloJexer(in, out);
            app.run(); // Blocks until user exits the TUI.
            if (exitCallback != null) {
                exitCallback.onExit(0);
            }
        } catch (Exception e) {
            if (exitCallback != null) {
                exitCallback.onExit(1, e.getMessage());
            }
        } finally {
            running = false;
            try {
                if (out != null) {
                    out.flush();
                }
            } catch (Exception ignore) {
                // Best effort flush.
            }
        }
    }
}
