package org.example;

import java.io.InputStream;
import java.io.OutputStream;
import java.util.ArrayList;
import java.util.List;

import jexer.TAction;
import jexer.TApplication;
import jexer.TLabel;
import jexer.TWindow;
import jexer.event.TKeypressEvent;
import jexer.event.TMenuEvent;
import jexer.event.TMouseEvent;
import jexer.menu.TMenu;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class HelloJexer extends TApplication {

    private static final Logger logger = LoggerFactory.getLogger(HelloJexer.class);
    
    private TWindow cvWindow;
    private final List<TLabel> labelWidgets = new ArrayList<>();
    private int scrollOffset = 0;
    private String[] contentLines;
    private int maxVisibleLines;
    private final int textStartY = 1;

    public HelloJexer() throws Exception {
        super(BackendType.SWING); // Use Swing backend when launching locally.
        buildUi();
    }

    /**
     * Construct a Jexer app that renders to an ANSI/xterm stream. This is
     * used by the SSH server to draw directly in the SSH client's terminal.
     */
    public HelloJexer(InputStream in, OutputStream out) throws Exception {
        logger.info("HelloJexer constructor called with InputStream: {}, OutputStream: {}", 
            in != null ? in.getClass().getName() : "null",
            out != null ? out.getClass().getName() : "null");
        super(in, out); // XTERM backend that speaks ANSI escape sequences.
        logger.info("HelloJexer super() call completed, building UI...");
        buildUi();
        logger.info("HelloJexer UI built successfully");
    }
    
    private String matrixText(String text) {
        return text;  // Plain text, no ANSI codes
    }
    
    private String matrixHeader(String text) {
        return text;  // Plain text, no ANSI codes
    }
    

    private void buildUi() {
        // Use full terminal size dynamically
        int screenWidth = getScreen().getWidth();
        int screenHeight = getScreen().getHeight();
        
        // Calculate available space (accounting for window borders and button)
        int windowWidth = screenWidth;
        int windowHeight = screenHeight;
        maxVisibleLines = windowHeight - 4; // Account for borders and button space

        // Matrix-themed main CV window - make it scrollable, use full screen
        cvWindow = addWindow("╔═══ THE MATRIX CV SYSTEM ═══╗", 0, 0, windowWidth, windowHeight);
        
        // Build complete CV content as a single scrollable text block
        StringBuilder cvContent = new StringBuilder();
        
        // Matrix ASCII Art Header with box-drawing characters
        cvContent.append(matrixHeader("╔═══════════════════════════════════════════════════════════════════════════════╗\n"));
        cvContent.append(matrixHeader("║                                                                               ║\n"));
        cvContent.append(matrixHeader("║    ███╗   ███╗ █████╗ ██████╗ ██████╗ ██╗██╗  ██╗                            ║\n"));
        cvContent.append(matrixHeader("║    ████╗ ████║██╔══██╗██╔══██╗██╔══██╗██║╚██╗██╔╝                            ║\n"));
        cvContent.append(matrixHeader("║    ██╔████╔██║███████║██████╔╝██████╔╝██║ ╚███╔╝                             ║\n"));
        cvContent.append(matrixHeader("║    ██║╚██╔╝██║██╔══██║██╔══██╗██╔══██╗██║ ██╔██╗                             ║\n"));
        cvContent.append(matrixHeader("║    ██║ ╚═╝ ██║██║  ██║██║  ██║██║  ██║██║██╔╝ ██╗                            ║\n"));
        cvContent.append(matrixHeader("║    ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝╚═╝  ╚═╝                            ║\n"));
        cvContent.append(matrixHeader("║                                                                               ║\n"));
        cvContent.append(matrixHeader("╚═══════════════════════════════════════════════════════════════════════════════╝\n"));
        cvContent.append("\n\n");
        
        // Name and Title with better spacing
        cvContent.append(matrixHeader("════════════════════════════════════════════════════════════════════════════════\n"));
        cvContent.append("\n");
        cvContent.append(matrixHeader("  ██████╗ ██╗██╗   ██╗ █████╗ ███╗   ██╗\n"));
        cvContent.append(matrixHeader("  ██╔══██╗██║╚██╗ ██╔╝██╔══██╗████╗  ██║\n"));
        cvContent.append(matrixHeader("  ██████╔╝██║ ╚████╔╝ ███████║██╔██╗ ██║\n"));
        cvContent.append(matrixHeader("  ██╔══██╗██║  ╚██╔╝  ██╔══██║██║╚██╗██║\n"));
        cvContent.append(matrixHeader("  ██║  ██║██║   ██║   ██║  ██║██║ ╚████║\n"));
        cvContent.append(matrixHeader("  ╚═╝  ╚═╝╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═══╝\n"));
        cvContent.append("\n");
        cvContent.append(matrixHeader("  SOFTWARE ENGINEER\n"));
        cvContent.append("\n\n");
        
        // Contact Information with better spacing
        cvContent.append(matrixText("  📧 Email    : riyankhanpyrex01@gmail.com\n"));
        cvContent.append(matrixText("  📱 Phone    : +91-7304100368\n"));
        cvContent.append(matrixText("  💼 LinkedIn : riyan--khan\n"));
        cvContent.append(matrixText("  🌐 Website  : pyrex01.github.io/Pyrex01/\n"));
        cvContent.append(matrixText("  🔗 GitHub   : Pyrex01\n"));
        cvContent.append(matrixText("  📍 Location : Mumbai, India\n"));
        cvContent.append("\n");
        cvContent.append(matrixHeader("════════════════════════════════════════════════════════════════════════════════\n"));
        cvContent.append("\n\n");
        
        // Summary
        cvContent.append(matrixHeader("  ╔═══ SUMMARY ═══╗\n"));
        cvContent.append(matrixHeader("  ╚═══════════════╝\n"));
        cvContent.append("\n");
        cvContent.append(matrixText("  Java and Node.js Developer with 3+ years of experience designing and deploying\n"));
        cvContent.append(matrixText("  scalable microservices and cloud-native applications. Proficient in Java, Spring\n"));
        cvContent.append(matrixText("  Boot, RESTful APIs, and cloud platforms like AWS. Experienced in CI/CD,\n"));
        cvContent.append(matrixText("  containerization, and agile workflows. Strong in data structures, algorithms,\n"));
        cvContent.append(matrixText("  and system design. Passionate about building efficient and reliable backend systems.\n"));
        cvContent.append("\n\n");
        
        // Skills
        cvContent.append(matrixHeader("  ╔═══ SKILLS ═══╗\n"));
        cvContent.append(matrixHeader("  ╚══════════════╝\n"));
        cvContent.append("\n");
        cvContent.append(matrixText("  • Languages  : Java, Node.js, Rust, SQL, Bash\n"));
        cvContent.append(matrixText("  • Frameworks : Spring Boot, Spring WebFlux, NestJS\n"));
        cvContent.append(matrixText("  • Databases  : MySQL, PostgreSQL\n"));
        cvContent.append(matrixText("  • Dev-Ops    : Git, Docker, Kubernetes, CI/CD, AWS (S3, EC2, RDS, CloudWatch, Lambda)\n"));
        cvContent.append("\n\n");
        
        // Projects
        cvContent.append(matrixHeader("  ╔═══ PROJECTS ═══╗\n"));
        cvContent.append(matrixHeader("  ╚═════════════════╝\n"));
        cvContent.append("\n");
        
        cvContent.append(matrixHeader("  [01] KRYPTORIA - Blockchain-based App\n"));
        cvContent.append(matrixText("       • Built and optimized backend infrastructure using Node.js, Express.js,\n"));
        cvContent.append(matrixText("         and Spring Boot, supporting dynamic in-game NFT asset transactions.\n"));
        cvContent.append(matrixText("       • Designed and developed wallet connection module, integrating with major\n"));
        cvContent.append(matrixText("         cryptocurrency wallets for secure login, asset management, and blockchain\n"));
        cvContent.append(matrixText("         interactions.\n"));
        cvContent.append("\n");
        
        cvContent.append(matrixHeader("  [02] WRKTALK - Real-time Chat Application\n"));
        cvContent.append(matrixText("       • Developed cross-platform frontend and backend features enabling secure\n"));
        cvContent.append(matrixText("         retrieval and synchronization of historical messages after new installs.\n"));
        cvContent.append(matrixText("       • Architected and implemented real-time messaging with Socket.IO, providing\n"));
        cvContent.append(matrixText("         reliable, low-latency communication for group and private chats.\n"));
        cvContent.append("\n");
        
        cvContent.append(matrixHeader("  [03] BBPS INTEGRATION - Payment Gateway\n"));
        cvContent.append(matrixText("       • Implemented secure payment workflows with instant digital receipt generation\n"));
        cvContent.append(matrixText("         and real-time transaction confirmations.\n"));
        cvContent.append(matrixText("       • Designed and developed backend modules for bill fetching, validation, and\n"));
        cvContent.append(matrixText("         payment processing, ensuring interoperability with multiple biller categories.\n"));
        cvContent.append(matrixText("       • Collaborated with frontend engineers to provide user-friendly bill payment\n"));
        cvContent.append(matrixText("         dashboard.\n"));
        cvContent.append("\n");
        
        cvContent.append(matrixHeader("  [04] ABRA DEFI - Bridge, Trading & Payments\n"));
        cvContent.append(matrixText("       • Engineered reliable USD to USDC on-ramping and off-ramping functionality.\n"));
        cvContent.append(matrixText("       • Integrated Rails.io for high-throughput crypto payments and settlement.\n"));
        cvContent.append(matrixText("       • Contributed to Talos trading integration for trade execution workflows.\n"));
        cvContent.append(matrixText("       • Collaborated with blockchain teams for accurate trade lifecycle handling.\n"));
        cvContent.append("\n");
        
        cvContent.append(matrixHeader("  [05] ABRA-FI - Solana + Spring WebFlux\n"));
        cvContent.append(matrixText("       • Spearheaded end-to-end backend development, integrating Solana blockchain\n"));
        cvContent.append(matrixText("         with reactive microservices.\n"));
        cvContent.append(matrixText("       • Integrated Solana SDK/RPC with Spring WebFlux for non-blocking smart\n"));
        cvContent.append(matrixText("         contract interactions.\n"));
        cvContent.append(matrixText("       • Implemented Solana program invocation flows: transaction creation, signing,\n"));
        cvContent.append(matrixText("         submission, and confirmation handling.\n"));
        cvContent.append(matrixText("       • Built blockchain crawlers for continuous on-chain program data processing.\n"));
        cvContent.append(matrixText("       • Designed reactive pipelines using Project Reactor for high-volume blockchain\n"));
        cvContent.append(matrixText("         events with low latency and back-pressure control.\n"));
        cvContent.append("\n\n");
        
        // Personal Project
        cvContent.append(matrixHeader("  ╔═══ PERSONAL PROJECT ═══╗\n"));
        cvContent.append(matrixHeader("  ╚═════════════════════════╝\n"));
        cvContent.append("\n");
        cvContent.append(matrixHeader("  WALKIE-TALKIE APP - Rust + Android\n"));
        cvContent.append(matrixText("       • Developed custom Rust library for low-level, CPU-efficient audio processing,\n"));
        cvContent.append(matrixText("         integrated with native Android.\n"));
        cvContent.append(matrixText("       • Designed system to capture audio from microphone, encode it, and transmit\n"));
        cvContent.append(matrixText("         through UDP over Ethernet in real time.\n"));
        cvContent.append("\n\n");
        
        // Professional Experience
        cvContent.append(matrixHeader("  ╔═══ PROFESSIONAL EXPERIENCE ═══╗\n"));
        cvContent.append(matrixHeader("  ╚═════════════════════════════════╝\n"));
        cvContent.append("\n");
        cvContent.append(matrixHeader("  SOFTWARE ENGINEER - Rejolut Solutions Pvt Ltd\n"));
        cvContent.append(matrixText("  📅 May 2022 – Present | 📍 India\n"));
        cvContent.append("\n");
        cvContent.append(matrixText("       • Designed and developed RESTful APIs using Spring Boot and Node.js for\n"));
        cvContent.append(matrixText("         enterprise applications, collaborating with cross-functional teams.\n"));
        cvContent.append(matrixText("       • Integrated containerization workflows with Docker and managed orchestration\n"));
        cvContent.append(matrixText("         using Kubernetes for robust, scalable deployment.\n"));
        cvContent.append(matrixText("       • Automated deployment pipelines with GitHub Actions, ensuring smooth CI/CD\n"));
        cvContent.append(matrixText("         processes and rapid delivery of new features.\n"));
        cvContent.append(matrixText("       • Regularly participated in code reviews and Agile sprint planning.\n"));
        cvContent.append("\n\n");
        
        // Education
        cvContent.append(matrixHeader("  ╔═══ EDUCATION ═══╗\n"));
        cvContent.append(matrixHeader("  ╚══════════════════╝\n"));
        cvContent.append("\n");
        cvContent.append(matrixText("  🎓 Bachelor's of Science in Information and Technology\n"));
        cvContent.append(matrixText("     Kalsekar Degree College | 2019 – 2022\n"));
        cvContent.append("\n");
        cvContent.append(matrixText("  🎓 Masters in Computer Application\n"));
        cvContent.append(matrixText("     Lovely Professional University | 2022 – 2026\n"));
        cvContent.append("\n\n");
        
        // Hobbies
        cvContent.append(matrixHeader("  ╔═══ HOBBIES ═══╗\n"));
        cvContent.append(matrixHeader("  ╚════════════════╝\n"));
        cvContent.append("\n");
        cvContent.append(matrixText("  • Exploring ARM and IoT devices\n"));
        cvContent.append(matrixText("  • Interested in low-level languages (Rust, C, Go)\n"));
        cvContent.append(matrixText("  • Built custom network routing systems for Kubernetes environments\n"));
        cvContent.append("\n\n");
        
        // Footer
        cvContent.append(matrixHeader("════════════════════════════════════════════════════════════════════════════════\n"));
        cvContent.append("\n");
        cvContent.append(matrixText("  [Use J/K or ↑/↓ to scroll | Mouse wheel to scroll | Press 'Q' or 'Escape' to exit] | Welcome to the Matrix...\n"));
        cvContent.append("\n");
        
        // Store content lines for scrolling
        contentLines = cvContent.toString().split("\n");
        
        // Exit button at bottom
        cvWindow.addButton("&Exit Matrix", windowWidth - 22, windowHeight - 3,
            new TAction() {
                @Override
                public void DO() {
                    exit();
                }
            }
        );
        
        // Initial render
        updateScrollDisplay();
    }
    
    private void updateScrollDisplay() {
        if (contentLines == null || cvWindow == null) {
            return;
        }
        
        // Ensure scroll offset is within bounds
        int totalLines = contentLines.length;
        int maxScroll = Math.max(0, totalLines - maxVisibleLines);
        scrollOffset = Math.max(0, Math.min(scrollOffset, maxScroll));
        
        // Update or create labels for visible lines
        int endLine = Math.min(scrollOffset + maxVisibleLines, totalLines);
        int visibleCount = endLine - scrollOffset;
        int maxWidth = cvWindow.getWidth() - 4;
        
        // Ensure we have enough labels
        while (labelWidgets.size() < visibleCount) {
            int yPos = textStartY + labelWidgets.size();
            TLabel label = cvWindow.addLabel("", 2, yPos);
            labelWidgets.add(label);
        }
        
        // Update label text for visible content
        for (int i = 0; i < visibleCount && i < labelWidgets.size(); i++) {
            int lineIndex = scrollOffset + i;
            if (lineIndex < contentLines.length) {
                String line = contentLines[lineIndex];
                // Truncate line if too long for window
                if (line.length() > maxWidth) {
                    line = line.substring(0, maxWidth);
                }
                labelWidgets.get(i).setLabel(line);
            }
        }
        
        // Hide extra labels by setting them to empty
        for (int i = visibleCount; i < labelWidgets.size(); i++) {
            labelWidgets.get(i).setLabel("");
        }
    }
    
    private void scrollDown(int lines) {
        scrollOffset += lines;
        updateScrollDisplay();
    }
    
    private void scrollUp(int lines) {
        scrollOffset = Math.max(0, scrollOffset - lines);
        updateScrollDisplay();
    }

    // Handle keyboard events for J/K scrolling
    @Override
    public boolean onKeypress(TKeypressEvent keypress) {
        logger.debug("onKeypress() called - keypress event received!");
        try {
            int keyCode = keypress.getKey().getKeyCode();
            logger.debug("Key code: 0x{} (decimal: {})", Integer.toHexString(keyCode), keyCode);
            
            // Check if it's a printable character (ASCII range)
            if (keyCode >= 32 && keyCode <= 126) {
                char ch = (char) keyCode;
                logger.info("Keypress detected: '{}' (0x{})", ch, Integer.toHexString(keyCode));
                
                // Handle J/K keys for nvim-style scrolling
                switch (ch) {
                    case 'j':
                    case 'J':
                        logger.info("Scrolling DOWN (J key pressed)");
                        scrollDown(1);
                        return true;
                    case 'k':
                    case 'K':
                        logger.info("Scrolling UP (K key pressed)");
                        scrollUp(1);
                        return true;
                    case 'q':
                    case 'Q':
                        logger.info("Exit requested (Q key pressed)");
                        exit();
                        return true;
                    default:
                        break;
                }
            }
        } catch (Exception e) {
            logger.warn("Error processing keypress", e);
        }
        
        // Try to handle special keys by key code
        int keyCode = keypress.getKey().getKeyCode();
        logger.debug("Checking special key codes: 0x{}", Integer.toHexString(keyCode));
        switch (keyCode) {
            case 0x102:
            case 0x148: // Down arrow
                logger.info("Scrolling DOWN (arrow key)");
                scrollDown(1);
                return true;
            case 0x103:
            case 0x151: // Up arrow
                logger.info("Scrolling UP (arrow key)");
                scrollUp(1);
                return true;
            case 0x149: // Page Up
                logger.info("Scrolling UP (Page Up)");
                scrollUp(maxVisibleLines / 2);
                return true;
            case 0x153: // Page Down
                logger.info("Scrolling DOWN (Page Down)");
                scrollDown(maxVisibleLines / 2);
                return true;
            default:
                logger.debug("Unhandled key code: 0x{}", Integer.toHexString(keyCode));
                break;
        }
        
        logger.debug("Calling super.onKeypress()");
        return super.onKeypress(keypress);
    }
    
    // Handle mouse events for scroll wheel - check if method exists
    public boolean onMouseEvent(TMouseEvent mouse) {
        // Handle mouse wheel scrolling - check event type
        TMouseEvent.Type type = mouse.getType();
        String typeStr = type.toString();
        if (typeStr.contains("WHEEL") || typeStr.contains("SCROLL")) {
            if (typeStr.contains("UP")) {
                scrollUp(3);
                return true;
            } else if (typeStr.contains("DOWN")) {
                scrollDown(3);
                return true;
            }
        }
        return false;
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