use ratatui::{
    prelude::*,
    text::{Line, Span},
    widgets::Paragraph,
};

// Helper function to create progress bar visual
fn create_progress_bar(filled: u8, total: u8, width: usize) -> String {
    let filled_chars = ((filled as f32 / total as f32) * width as f32) as usize;
    let empty_chars = width.saturating_sub(filled_chars);
    format!(
        "{}{}",
        "█".repeat(filled_chars.min(width)),
        "░".repeat(empty_chars)
    )
}

// Build all resume lines and return them (used for both counting and rendering)
pub fn build_resume_lines(area: Rect) -> Vec<Line<'static>> {
    // Enhanced btop-inspired color scheme with vibrant accents
    let accent_cyan = Color::LightCyan;
    let accent_green = Color::LightGreen;
    let accent_yellow = Color::LightYellow;
    let accent_magenta = Color::LightMagenta;
    let accent_blue = Color::LightBlue;
    let text_color = Color::White;
    let dim_color = Color::Gray;
    
    // Enhanced styles with better visual hierarchy
    let border_style = Style::default()
        .fg(accent_cyan)
        .add_modifier(ratatui::style::Modifier::BOLD);
    let title_style = Style::default()
        .fg(accent_cyan)
        .add_modifier(ratatui::style::Modifier::BOLD | ratatui::style::Modifier::UNDERLINED);
    let section_style = Style::default()
        .fg(accent_green)
        .add_modifier(ratatui::style::Modifier::BOLD);
    let highlight_style = Style::default()
        .fg(accent_yellow)
        .add_modifier(ratatui::style::Modifier::BOLD);
    let normal_style = Style::default().fg(text_color);
    let dim_style = Style::default().fg(dim_color);
    let project_style = Style::default()
        .fg(accent_magenta)
        .add_modifier(ratatui::style::Modifier::BOLD);
    let link_style = Style::default()
        .fg(accent_blue)
        .add_modifier(ratatui::style::Modifier::UNDERLINED);
    
    // Build all lines with proper styling
    let mut lines = Vec::new();
    
    // Calculate border width based on terminal width
    let border_width = area.width.saturating_sub(2);
    let border_line = "═".repeat(border_width as usize);
    let border_start = "╔";
    let border_end = "╗";
    let border_mid = "╠";
    let border_side = "║";
    let border_bottom_start = "╚";
    let border_bottom_end = "╝";
    
    // Helper macro to create styled lines
    macro_rules! add_line {
        ($($span:expr),*) => {
            lines.push(Line::from(vec![$($span),*]));
        };
    }
    
    macro_rules! styled_str {
        ($fmt:expr, $style:expr) => {
            Span::styled($fmt.to_string(), $style)
        };
    }
    
    // Enhanced header with visual effects
    let top_border = format!("{}{}{}", border_start, border_line, border_end);
    add_line!(Span::styled("", normal_style));
    add_line!(styled_str!(top_border, border_style));
    
    // Header with gradient-like effect
    let name_padding = (border_width as usize).saturating_sub(30) / 2;
    lines.push(Line::from(vec![
        Span::styled(border_side, border_style),
        Span::styled("  ", normal_style),
        Span::styled(" ".repeat(name_padding), normal_style),
        Span::styled("RIYAN KHAN", title_style),
        Span::styled("  ", normal_style),
        Span::styled("◆", accent_cyan),
        Span::styled("  ", normal_style),
        Span::styled("Software Engineer", highlight_style),
    ]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Navigation help text
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2050_17 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_17, normal_style),
        Span::styled("⌨  Navigation: ", dim_style),
        Span::styled("[↑/↓]", accent_cyan),
        Span::styled(" or ", dim_style),
        Span::styled("[j/k]", accent_cyan),
        Span::styled(" Scroll  ", dim_style),
        Span::styled("[Home/End]", accent_cyan),
        Span::styled(" or ", dim_style),
        Span::styled("[g/G]", accent_cyan),
        Span::styled(" Jump  ", dim_style),
        Span::styled("[Q]", accent_yellow),
        Span::styled(" Quit", dim_style),
    ]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Contact information with icons
    let _s3389 = format!("{}  📧 CONTACT INFORMATION", border_side);
    lines.push(Line::from(vec![Span::styled(_s3389, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    
    let _s7089 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s7089, normal_style),
        Span::styled("✉ ", accent_blue),
        Span::styled("Email: ", highlight_style),
        Span::styled("riyankhanpyrex01@gmail.com", link_style),
    ]));
    
    let _s872 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s872, normal_style),
        Span::styled("📱 ", accent_blue),
        Span::styled("Phone: ", highlight_style),
        Span::styled("+91-7304100368", normal_style),
    ]));
    
    let _s3327 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s3327, normal_style),
        Span::styled("💼 ", accent_blue),
        Span::styled("LinkedIn: ", highlight_style),
        Span::styled("riyan--khan", link_style),
    ]));
    
    let _s1384 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s1384, normal_style),
        Span::styled("🌐 ", accent_blue),
        Span::styled("Website: ", highlight_style),
        Span::styled("pyrex01.github.io/Pyrex01/", link_style),
    ]));
    
    let _s7200 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s7200, normal_style),
        Span::styled("🐙 ", accent_blue),
        Span::styled("GitHub: ", highlight_style),
        Span::styled("Pyrex01", link_style),
    ]));
    
    let _s2380 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2380, normal_style),
        Span::styled("📍 ", accent_blue),
        Span::styled("Location: ", highlight_style),
        Span::styled("Mumbai, India", normal_style),
    ]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Summary with icon
    let _s9799 = format!("{}  📋 SUMMARY", border_side);
    lines.push(Line::from(vec![Span::styled(_s9799, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s8815 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s8815, normal_style),
        Span::styled("Java and Node.js Developer with ", normal_style),
        Span::styled("3+ years", highlight_style),
        Span::styled(" of experience", normal_style),
    ]));
    let _s9348 = format!("{}  designing and deploying scalable microservices and", border_side);
    lines.push(Line::from(vec![Span::styled(_s9348, normal_style)]));
    let _s9509 = format!("{}  cloud-native applications. Proficient in ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s9509, normal_style),
        Span::styled("Java, Spring Boot, RESTful APIs", highlight_style),
        Span::styled(", and cloud platforms like AWS.", normal_style),
    ]));
    let _s9026 = format!("{}  Experienced in CI/CD, containerization, and agile", border_side);
    lines.push(Line::from(vec![Span::styled(_s9026, normal_style)]));
    let _s6881 = format!("{}  workflows. Strong in data structures, algorithms, and", border_side);
    lines.push(Line::from(vec![Span::styled(_s6881, normal_style)]));
    let _s8766 = format!("{}  system design. Passionate about building efficient and", border_side);
    lines.push(Line::from(vec![Span::styled(_s8766, normal_style)]));
    let _s5206 = format!("{}  reliable backend systems.", border_side);
    lines.push(Line::from(vec![Span::styled(_s5206, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Skills with progress bars (btop-style)
    let _s4406 = format!("{}  ⚡ SKILLS", border_side);
    lines.push(Line::from(vec![Span::styled(_s4406, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    
    // Languages with progress bars
    let _s2050 = format!("{}  ", border_side);
    let lang_bar_width = (border_width as usize).saturating_sub(25).min(30);
    lines.push(Line::from(vec![
        Span::styled(_s2050, normal_style),
        Span::styled("🔷 Languages: ", highlight_style),
        Span::styled("Java, Node.js, Rust, SQL, Bash", normal_style),
    ]));
    let _s_lang_bar = format!("{}  ", border_side);
    let lang_progress = create_progress_bar(9, 10, lang_bar_width);
    lines.push(Line::from(vec![
        Span::styled(_s_lang_bar, normal_style),
        Span::styled("    ", normal_style),
        Span::styled(lang_progress, Style::default().fg(accent_green)),
        Span::styled(" 95%", Style::default().fg(accent_green).add_modifier(ratatui::style::Modifier::BOLD)),
    ]));
    
    // Frameworks
    let _s2050_2 = format!("{}  ", border_side);
    let framework_bar_width = (border_width as usize).saturating_sub(25).min(30);
    lines.push(Line::from(vec![
        Span::styled(_s2050_2, normal_style),
        Span::styled("🔷 Frameworks: ", highlight_style),
        Span::styled("Spring Boot, Spring WebFlux, NestJS", normal_style),
    ]));
    let _s_framework_bar = format!("{}  ", border_side);
    let framework_progress = create_progress_bar(9, 10, framework_bar_width);
    lines.push(Line::from(vec![
        Span::styled(_s_framework_bar, normal_style),
        Span::styled("    ", normal_style),
        Span::styled(framework_progress, Style::default().fg(accent_cyan)),
        Span::styled(" 92%", Style::default().fg(accent_cyan).add_modifier(ratatui::style::Modifier::BOLD)),
    ]));
    
    // Databases
    let _s2050_3 = format!("{}  ", border_side);
    let db_bar_width = (border_width as usize).saturating_sub(25).min(30);
    lines.push(Line::from(vec![
        Span::styled(_s2050_3, normal_style),
        Span::styled("🔷 Databases: ", highlight_style),
        Span::styled("MySQL, PostgreSQL", normal_style),
    ]));
    let _s_db_bar = format!("{}  ", border_side);
    let db_progress = create_progress_bar(8, 10, db_bar_width);
    lines.push(Line::from(vec![
        Span::styled(_s_db_bar, normal_style),
        Span::styled("    ", normal_style),
        Span::styled(db_progress, Style::default().fg(accent_yellow)),
        Span::styled(" 88%", Style::default().fg(accent_yellow).add_modifier(ratatui::style::Modifier::BOLD)),
    ]));
    
    // DevOps
    let _s2050_7 = format!("{}  ", border_side);
    let devops_bar_width = (border_width as usize).saturating_sub(25).min(30);
    lines.push(Line::from(vec![
        Span::styled(_s2050_7, normal_style),
        Span::styled("🔷 Dev-Ops: ", highlight_style),
        Span::styled("Git, Docker, Kubernetes, CI/CD, AWS", normal_style),
    ]));
    let _s_devops_bar = format!("{}  ", border_side);
    let devops_progress = create_progress_bar(9, 10, devops_bar_width);
    lines.push(Line::from(vec![
        Span::styled(_s_devops_bar, normal_style),
        Span::styled("    ", normal_style),
        Span::styled(devops_progress, Style::default().fg(accent_magenta)),
        Span::styled(" 94%", Style::default().fg(accent_magenta).add_modifier(ratatui::style::Modifier::BOLD)),
    ]));
    let _s_devops = format!("{}              (S3, EC2, RDS, CloudWatch, Lambda)", border_side);
    lines.push(Line::from(vec![Span::styled(_s_devops, dim_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Projects with enhanced visuals
    let _s255 = format!("{}  🚀 PROJECTS", border_side);
    lines.push(Line::from(vec![Span::styled(_s255, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2050_8 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_8, normal_style),
        Span::styled("▶ ", accent_cyan),
        Span::styled("Kryptoria", project_style),
        Span::styled(" - Blockchain-based app", normal_style),
    ]));
    let _s6702 = format!("{}    • Built and optimized backend infrastructure using", border_side);
    lines.push(Line::from(vec![Span::styled(_s6702, normal_style)]));
    let _s762 = format!("{}      Node.js, Express.js, and Spring Boot, supporting", border_side);
    lines.push(Line::from(vec![Span::styled(_s762, normal_style)]));
    let _s4539 = format!("{}      dynamic in-game NFT asset transactions.", border_side);
    lines.push(Line::from(vec![Span::styled(_s4539, normal_style)]));
    let _s2080 = format!("{}    • Designed and developed the wallet connection module,", border_side);
    lines.push(Line::from(vec![Span::styled(_s2080, normal_style)]));
    let _s6298 = format!("{}      integrating with major cryptocurrency wallets for", border_side);
    lines.push(Line::from(vec![Span::styled(_s6298, normal_style)]));
    let _s6153 = format!("{}      secure user login and asset management.", border_side);
    lines.push(Line::from(vec![Span::styled(_s6153, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    
    let _s2050_9 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_9, normal_style),
        Span::styled("▶ ", accent_cyan),
        Span::styled("Wrktalk", project_style),
        Span::styled(" - Real-time chat application", normal_style),
    ]));
    let _s8189 = format!("{}    • Developed cross-platform frontend and backend features", border_side);
    lines.push(Line::from(vec![Span::styled(_s8189, normal_style)]));
    let _s5344 = format!("{}      enabling secure retrieval and synchronization of", border_side);
    lines.push(Line::from(vec![Span::styled(_s5344, normal_style)]));
    let _s1612 = format!("{}      historical messages after new installs.", border_side);
    lines.push(Line::from(vec![Span::styled(_s1612, normal_style)]));
    let _s4278 = format!("{}    • Architected and implemented real-time messaging with", border_side);
    lines.push(Line::from(vec![Span::styled(_s4278, normal_style)]));
    let _s3876 = format!("{}      Socket.IO for reliable, low-latency communication.", border_side);
    lines.push(Line::from(vec![Span::styled(_s3876, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    
    let _s2050_10 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_10, normal_style),
        Span::styled("▶ ", accent_cyan),
        Span::styled("BBPS Integration", project_style),
        Span::styled(" - Payment gateway", normal_style),
    ]));
    let _s5735 = format!("{}    • Implemented secure payment workflows with instant", border_side);
    lines.push(Line::from(vec![Span::styled(_s5735, normal_style)]));
    let _s3634 = format!("{}      digital receipt generation and real-time transaction", border_side);
    lines.push(Line::from(vec![Span::styled(_s3634, normal_style)]));
    let _s6958 = format!("{}      confirmations.", border_side);
    lines.push(Line::from(vec![Span::styled(_s6958, normal_style)]));
    let _s1627 = format!("{}    • Designed backend modules for bill fetching, validation,", border_side);
    lines.push(Line::from(vec![Span::styled(_s1627, normal_style)]));
    let _s202 = format!("{}      and payment processing.", border_side);
    lines.push(Line::from(vec![Span::styled(_s202, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    
    let _s2050_11 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_11, normal_style),
        Span::styled("▶ ", accent_cyan),
        Span::styled("Abra DeFi", project_style),
        Span::styled(" - Bridge, Trading & Payments", normal_style),
    ]));
    let _s4808 = format!("{}    • Engineered reliable USD to USDC on-ramping and", border_side);
    lines.push(Line::from(vec![Span::styled(_s4808, normal_style)]));
    let _s9780 = format!("{}      off-ramping functionality for seamless fiat-to-crypto", border_side);
    lines.push(Line::from(vec![Span::styled(_s9780, normal_style)]));
    let _s1211 = format!("{}      transactions.", border_side);
    lines.push(Line::from(vec![Span::styled(_s1211, normal_style)]));
    let _s3530 = format!("{}    • Integrated Rails.io for high-throughput crypto payments.", border_side);
    lines.push(Line::from(vec![Span::styled(_s3530, normal_style)]));
    let _s0 = format!("{}    • Contributed to Talos trading integration.", border_side);
    lines.push(Line::from(vec![Span::styled(_s0, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    
    let _s2050_4 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_4, normal_style),
        Span::styled("▶ ", accent_cyan),
        Span::styled("Abra-Fi", project_style),
        Span::styled(" - Solana + Spring WebFlux", normal_style),
    ]));
    let _s702 = format!("{}    • Spearheaded end-to-end backend development, integrating", border_side);
    lines.push(Line::from(vec![Span::styled(_s702, normal_style)]));
    let _s9277 = format!("{}      Solana blockchain capabilities with reactive microservices.", border_side);
    lines.push(Line::from(vec![Span::styled(_s9277, normal_style)]));
    let _s787 = format!("{}    • Integrated Solana SDK/RPC with Spring WebFlux for", border_side);
    lines.push(Line::from(vec![Span::styled(_s787, normal_style)]));
    let _s3806 = format!("{}      non-blocking smart contract interactions.", border_side);
    lines.push(Line::from(vec![Span::styled(_s3806, normal_style)]));
    let _s7625 = format!("{}    • Built blockchain crawlers to continuously fetch and", border_side);
    lines.push(Line::from(vec![Span::styled(_s7625, normal_style)]));
    let _s1524 = format!("{}      process on-chain program data.", border_side);
    lines.push(Line::from(vec![Span::styled(_s1524, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Personal Project
    let _s7097 = format!("{}  💻 PERSONAL PROJECT", border_side);
    lines.push(Line::from(vec![Span::styled(_s7097, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2050_12 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_12, normal_style),
        Span::styled("▶ ", accent_cyan),
        Span::styled("Walkie-Talkie App", project_style),
        Span::styled(" - Rust + Android", normal_style),
    ]));
    let _s2057 = format!("{}    • Developed a custom Rust library for low-level,", border_side);
    lines.push(Line::from(vec![Span::styled(_s2057, normal_style)]));
    let _s7641 = format!("{}      CPU-efficient audio processing, integrated with", border_side);
    lines.push(Line::from(vec![Span::styled(_s7641, normal_style)]));
    let _s2680 = format!("{}      native Android.", border_side);
    lines.push(Line::from(vec![Span::styled(_s2680, normal_style)]));
    let _s4607 = format!("{}    • Designed and implemented a system to capture audio,", border_side);
    lines.push(Line::from(vec![Span::styled(_s4607, normal_style)]));
    let _s646 = format!("{}      encode it, and transmit through UDP over Ethernet", border_side);
    lines.push(Line::from(vec![Span::styled(_s646, normal_style)]));
    let _s726 = format!("{}      in real time.", border_side);
    lines.push(Line::from(vec![Span::styled(_s726, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Professional Experience
    let _s6459 = format!("{}  💼 PROFESSIONAL EXPERIENCE", border_side);
    lines.push(Line::from(vec![Span::styled(_s6459, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2050_5 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_5, normal_style),
        Span::styled("Software Engineer", highlight_style),
        Span::styled("  ", normal_style),
        Span::styled("◆ ", accent_cyan),
        Span::styled("Rejolut Solutions Pvt Ltd", accent_cyan),
    ]));
    let _s2050_6 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_6, normal_style),
        Span::styled("May 2022 – Present | India", dim_style),
    ]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s7345 = format!("{}  • Designed and developed RESTful APIs using Spring Boot", border_side);
    lines.push(Line::from(vec![Span::styled(_s7345, normal_style)]));
    let _s5052 = format!("{}    and Node.js for enterprise applications.", border_side);
    lines.push(Line::from(vec![Span::styled(_s5052, normal_style)]));
    let _s5218 = format!("{}  • Integrated containerization workflows with Docker and", border_side);
    lines.push(Line::from(vec![Span::styled(_s5218, normal_style)]));
    let _s6084 = format!("{}    managed orchestration using Kubernetes.", border_side);
    lines.push(Line::from(vec![Span::styled(_s6084, normal_style)]));
    let _s8108 = format!("{}  • Automated deployment pipelines with GitHub Actions.", border_side);
    lines.push(Line::from(vec![Span::styled(_s8108, normal_style)]));
    let _s3550 = format!("{}  • Regularly participated in code reviews and Agile", border_side);
    lines.push(Line::from(vec![Span::styled(_s3550, normal_style)]));
    let _s1462 = format!("{}    sprint planning.", border_side);
    lines.push(Line::from(vec![Span::styled(_s1462, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Education
    let _s505 = format!("{}  🎓 EDUCATION", border_side);
    lines.push(Line::from(vec![Span::styled(_s505, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2050_13 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_13, normal_style),
        Span::styled("Bachelor's of Science in Information and Technology", highlight_style),
    ]));
    let _s2050_14 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_14, normal_style),
        Span::styled("Kalsekar Degree College", accent_cyan),
        Span::styled("  ", normal_style),
        Span::styled("2019 – 2022", dim_style),
    ]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2050_15 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_15, normal_style),
        Span::styled("Masters in Computer Application", highlight_style),
    ]));
    let _s2050_16 = format!("{}  ", border_side);
    lines.push(Line::from(vec![
        Span::styled(_s2050_16, normal_style),
        Span::styled("Lovely Professional University", accent_cyan),
        Span::styled("  ", normal_style),
        Span::styled("2022 – 2026", dim_style),
    ]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_mid, border_line, border_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    // Hobbies
    let _s5688 = format!("{}  🎯 HOBBIES", border_side);
    lines.push(Line::from(vec![Span::styled(_s5688, section_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s169 = format!("{}  • Exploring ARM and IoT devices", border_side);
    lines.push(Line::from(vec![Span::styled(_s169, normal_style)]));
    let _s_languages = format!("{}  • Interested in low-level languages (Rust, C, Go)", border_side);
    lines.push(Line::from(vec![Span::styled(_s_languages, normal_style)]));
    let _s7209 = format!("{}  • Built custom network routing systems for Kubernetes", border_side);
    lines.push(Line::from(vec![Span::styled(_s7209, normal_style)]));
    lines.push(Line::from(vec![Span::styled(border_side, border_style)]));
    let _s2708 = format!("{}{}{}", border_bottom_start, border_line, border_bottom_end);
    lines.push(Line::from(vec![Span::styled(_s2708, border_style)]));
    
    lines
}

// Render the resume UI with retro-futuristic styling
// Returns the total number of lines
pub fn render_resume_ui(buffer: &mut Buffer, area: Rect, scroll_offset: u16) -> usize {
    
    // Build all lines
    let lines = build_resume_lines(area);
    let total_lines = lines.len();
    
    // Render lines with scroll offset
    let start_y = scroll_offset as usize;
    let max_lines = area.height as usize;
    
    for (idx, line) in lines.iter().enumerate() {
        let y_pos = idx.saturating_sub(start_y);
        if y_pos < max_lines && idx >= start_y {
            let paragraph = Paragraph::new(line.clone());
            paragraph.render(Rect::new(0, y_pos as u16, area.width, 1), buffer);
        }
    }
    
    total_lines
}
