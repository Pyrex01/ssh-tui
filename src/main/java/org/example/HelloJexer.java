package org.example;

import java.io.InputStream;
import java.io.OutputStream;
import jexer.TAction;
import jexer.TApplication;
import jexer.TWindow;
import jexer.event.TMenuEvent;
import jexer.menu.TMenu;

public class HelloJexer extends TApplication {

    public HelloJexer() throws Exception {
        super(BackendType.SWING); // Use Swing backend when launching locally.
        buildUi(false);
    }

    /**
     * Construct a Jexer app that renders to an ANSI/xterm stream. This is
     * used by the SSH server to draw directly in the SSH client's terminal.
     */
    public HelloJexer(InputStream in, OutputStream out) throws Exception {
        super(in, out); // XTERM backend that speaks ANSI escape sequences.
        buildUi(true);
    }

    private void buildUi(boolean fullScreen) {
        int width = fullScreen ? Math.max(10, getScreen().getWidth() - 2) : 40;
        int height = fullScreen ? Math.max(5, getScreen().getHeight() - 2) : 10;

        // 1. Create a window
        TWindow helloWindow = addWindow("Riyan's View", 1, 1, width, height);

        // 2. Add a label (the "Hello, World!" text)
        helloWindow.addLabel("Hello, Cli World! This is Riyan here!", 2, 2);

        // 3. Add an exit button
        helloWindow.addButton("&Exit", 15, 5,
            new TAction() {
                @Override
                public void DO() {
                    // This action runs when the button is pressed
                    exit();
                }
            }
        );

    }

    // Optional: Override the menu setup if you want a menu bar
    @Override
    protected boolean onMenu(TMenuEvent menu) {
        if (menu.getId() == TMenu.MID_EXIT) {
            exit();
            return true;
        }
        return super.onMenu(menu);
    }
}