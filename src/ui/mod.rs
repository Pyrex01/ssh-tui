use ratatui::{
    prelude::*,
    text::{Line, Span},
    widgets::Paragraph,
};

fn repeat_pattern(pattern: &str, width: usize) -> String {
    if pattern.is_empty() || width == 0 {
        return String::new();
    }
    let mut out = String::with_capacity(width);
    while out.chars().count() < width {
        out.push_str(pattern);
    }
    out.chars().take(width).collect()
}

fn progress_bar(score: u8, width: usize) -> String {
    let filled = ((score as f32 / 10.0) * width as f32).round() as usize;
    format!(
        "{}{}",
        "█".repeat(filled.min(width)),
        "░".repeat(width.saturating_sub(filled))
    )
}

fn wrap_text(input: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    let mut out = Vec::new();
    let mut current = String::new();
    for word in input.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
            continue;
        }
        if current.len() + 1 + word.len() > width {
            out.push(current);
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

fn push_wrapped(lines: &mut Vec<Line<'static>>, prefix: &str, body: &str, style: Style, width: usize) {
    let body_width = width.saturating_sub(prefix.len());
    let wrapped = wrap_text(body, body_width.max(8));
    for (i, row) in wrapped.into_iter().enumerate() {
        let lead = if i == 0 {
            prefix.to_string()
        } else {
            " ".repeat(prefix.len())
        };
        lines.push(Line::from(Span::styled(format!("{lead}{row}"), style)));
    }
}

pub fn build_resume_lines(area: Rect) -> Vec<Line<'static>> {
    let width = area.width.max(40) as usize;
    let compact = width < 74;

    let sky = Style::default().fg(Color::LightBlue);
    let hero = Style::default()
        .fg(Color::LightYellow)
        .add_modifier(Modifier::BOLD);
    let accent = Style::default()
        .fg(Color::LightRed)
        .add_modifier(Modifier::BOLD);
    let section = Style::default()
        .fg(Color::LightGreen)
        .add_modifier(Modifier::BOLD);
    let text = Style::default().fg(Color::White);
    let dim = Style::default().fg(Color::Gray);
    let link = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::UNDERLINED);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let divider = repeat_pattern("=-", width);
    let ground = repeat_pattern("▓", width);

    lines.push(Line::from(Span::styled(repeat_pattern(" ", width), text)));
    lines.push(Line::from(vec![
        Span::styled("      ☁        ☁        ☁  ", sky),
        Span::styled("1UP PORTFOLIO", hero),
    ]));
    if compact {
        lines.push(Line::from(vec![
            Span::styled("  ▄▄▄ ", accent),
            Span::styled("RIYAN KHAN", hero),
            Span::styled(" | Backend Engineer", text),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("  ▄▄▄▄   ▄▄   ▄▄   ▄▄▄▄  ", accent),
            Span::styled("RIYAN KHAN", hero),
            Span::styled("  //  Software Engineer", text),
        ]));
    }
    lines.push(Line::from(vec![
        Span::styled("  HUD: ", dim),
        Span::styled("Coins 128", accent),
        Span::styled(" | World Mumbai-1", section),
        Span::styled(" | Mission: Build scalable systems", text),
    ]));
    lines.push(Line::from(Span::styled(divider.clone(), dim)));
    lines.push(Line::from(vec![
        Span::styled("Controls ", dim),
        Span::styled("[j/k] [↑/↓] scroll ", text),
        Span::styled("[g/G] home/end ", text),
        Span::styled("[q] quit", accent),
    ]));
    lines.push(Line::from(Span::styled(divider.clone(), dim)));

    lines.push(Line::from(Span::styled(">> PLAYER CARD", section)));
    lines.push(Line::from(vec![
        Span::styled("Name: ", dim),
        Span::styled("Riyan Khan", hero),
        Span::styled(" | Role: ", dim),
        Span::styled("Java + Node.js Developer", text),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Email: ", dim),
        Span::styled("riyankhanpyrex01@gmail.com", link),
    ]));
    lines.push(Line::from(vec![
        Span::styled("GitHub: ", dim),
        Span::styled("github.com/Pyrex01", link),
        Span::styled(" | LinkedIn: ", dim),
        Span::styled("riyan--khan", link),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Base: ", dim),
        Span::styled("Mumbai, India", text),
        Span::styled(" | Website: ", dim),
        Span::styled("pyrex01.github.io/Pyrex01/", link),
    ]));
    lines.push(Line::from(Span::styled(divider.clone(), dim)));

    lines.push(Line::from(Span::styled(">> MAIN QUEST", section)));
    push_wrapped(
        &mut lines,
        "  ",
        "Java and Node.js developer with 3+ years building backend services and cloud-native systems. Strong in Spring Boot, REST APIs, Docker, Kubernetes, CI/CD, and scalable architecture.",
        text,
        width,
    );
    lines.push(Line::from(Span::styled(divider.clone(), dim)));

    lines.push(Line::from(Span::styled(">> SKILL TREE", section)));
    let meter_width = width.saturating_sub(40).clamp(10, 30);
    let skills = [
        ("Languages  ", 9, "Java, Node.js, Rust, SQL, Bash"),
        ("Frameworks ", 9, "Spring Boot, WebFlux, NestJS"),
        ("Databases  ", 8, "MySQL, PostgreSQL"),
        ("DevOps     ", 9, "Docker, Kubernetes, AWS, CI/CD"),
    ];
    for (label, score, info) in skills {
        lines.push(Line::from(vec![
            Span::styled(format!("  {label} "), dim),
            Span::styled(progress_bar(score, meter_width), accent),
            Span::styled(format!("  {}/10", score), hero),
        ]));
        push_wrapped(&mut lines, "    ", info, text, width);
    }
    lines.push(Line::from(Span::styled(divider.clone(), dim)));

    lines.push(Line::from(Span::styled(">> BOSSES DEFEATED (PROJECTS)", section)));
    let projects = [
        (
            "Kryptoria",
            "Blockchain app with Node.js + Spring Boot backend and wallet integration for NFT flows.",
        ),
        (
            "Wrktalk",
            "Real-time chat platform with reliable message sync and Socket.IO low-latency delivery.",
        ),
        (
            "BBPS Integration",
            "Secure payment gateway modules for bill fetch, validation, receipts, and confirmations.",
        ),
        (
            "Abra DeFi",
            "Built core on/off-ramp and crypto payments integrations for USD <-> USDC operations.",
        ),
        (
            "Abra-Fi",
            "Solana-integrated reactive backend with Spring WebFlux and chain crawler pipelines.",
        ),
        (
            "Walkie-Talkie App",
            "Personal Rust + Android project for low-level audio processing and real-time UDP transport.",
        ),
    ];
    for (name, detail) in projects {
        lines.push(Line::from(vec![
            Span::styled("  ▶ ", accent),
            Span::styled(name, hero),
        ]));
        push_wrapped(&mut lines, "    ", detail, text, width);
    }
    lines.push(Line::from(Span::styled(divider.clone(), dim)));

    lines.push(Line::from(Span::styled(">> EXPERIENCE", section)));
    lines.push(Line::from(vec![
        Span::styled("  Software Engineer", hero),
        Span::styled(" @ Rejolut Solutions Pvt Ltd", text),
    ]));
    lines.push(Line::from(Span::styled("  May 2022 - Present", dim)));
    let exp_points = [
        "Designed and shipped RESTful APIs using Spring Boot and Node.js.",
        "Implemented Docker + Kubernetes based deployment workflows.",
        "Automated delivery with GitHub Actions and CI/CD pipelines.",
        "Contributed in reviews, sprint planning, and production hardening.",
    ];
    for point in exp_points {
        push_wrapped(&mut lines, "  • ", point, text, width);
    }
    lines.push(Line::from(Span::styled(divider.clone(), dim)));

    lines.push(Line::from(Span::styled(">> EDUCATION", section)));
    lines.push(Line::from(Span::styled(
        "  BSc IT - Kalsekar Degree College (2019 - 2022)",
        text,
    )));
    lines.push(Line::from(Span::styled(
        "  MCA - Lovely Professional University (2022 - 2026)",
        text,
    )));
    lines.push(Line::from(Span::styled(divider.clone(), dim)));

    lines.push(Line::from(Span::styled(">> SIDE QUESTS", section)));
    let hobbies = [
        "Exploring ARM and IoT devices",
        "Low-level systems in Rust, C, and Go",
        "Custom networking and Kubernetes routing experiments",
    ];
    for hobby in hobbies {
        push_wrapped(&mut lines, "  * ", hobby, text, width);
    }
    lines.push(Line::from(Span::styled(divider, dim)));
    lines.push(Line::from(vec![
        Span::styled("  PIPE EXIT ", accent),
        Span::styled("Thanks for visiting this terminal world.", text),
    ]));
    lines.push(Line::from(Span::styled(ground, Style::default().fg(Color::Green))));

    lines
}

pub fn render_resume_ui(buffer: &mut Buffer, area: Rect, scroll_offset: u16) -> usize {
    let lines = build_resume_lines(area);
    let total_lines = lines.len();
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
