package org.example;

import java.io.InputStream;
import java.io.OutputStream;
import org.apache.sshd.server.Environment;
import org.apache.sshd.server.ExitCallback;
import org.apache.sshd.server.channel.ChannelSession;
import org.apache.sshd.server.command.Command;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class CustomCommand implements Command, Runnable {

    private static final Logger logger = LoggerFactory.getLogger(CustomCommand.class);
    
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
        logger.info("CustomCommand constructor called for channel: {}", channel);
    }

    // ===== Stream setters =====

    @Override
    public void setInputStream(InputStream in) {
        logger.info("Setting InputStream - wrapping with LoggingInputStream for keystroke monitoring");
        this.in = new LoggingInputStream(in);
        logger.info("InputStream wrapped and ready to log keystrokes");
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
        logger.info("CustomCommand.start() called");
        logger.info("Environment: {}", env);
        if (env != null) {
            logger.info("Environment variables: {}", env.getEnv());
        }
        worker = new Thread(this, "ssh-lanterna-session");
        logger.info("Starting worker thread: {}", worker.getName());
        worker.start();
        logger.info("Worker thread started");
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
            logger.info("Starting HelloJexer application with InputStream logging enabled");
            logger.info("All keystrokes will be logged at DEBUG level");
            logger.info("To see keystroke logs, ensure logging level is set to DEBUG");
            
            if (in == null) {
                logger.error("InputStream is null! Keystrokes cannot be received.");
            } else {
                logger.info("InputStream is ready: {}", in.getClass().getName());
            }
            
            HelloJexer app = new HelloJexer(in, out);
            logger.info("HelloJexer created, starting application...");
            logger.info("Calling app.run() - this should block until user exits");
            try {
                app.run(); // Blocks until user exits the TUI.
                logger.info("HelloJexer application exited normally");
            } catch (Exception e) {
                logger.error("Exception during app.run()", e);
                throw e;
            }
            
            if (exitCallback != null) {
                exitCallback.onExit(0);
            }
        } catch (Exception e) {
            logger.error("Error running HelloJexer application", e);
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
