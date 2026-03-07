use ratatui::{
    prelude::*,
    text::{Line, Span},
    widgets::Paragraph,
};

fn wrap_text(input: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }

    let mut out = Vec::new();
    let mut line = String::new();
    for word in input.split_whitespace() {
        if line.is_empty() {
            line.push_str(word);
            continue;
        }

        if line.len() + 1 + word.len() > width {
            out.push(line);
            line = word.to_string();
        } else {
            line.push(' ');
            line.push_str(word);
        }
    }

    if !line.is_empty() {
        out.push(line);
    }
    if out.is_empty() {
        out.push(String::new());
    }

    out
}

fn push_wrapped(lines: &mut Vec<Line<'static>>, prefix: &str, text: &str, style: Style, width: usize) {
    let text_width = width.saturating_sub(prefix.len()).max(10);
    for (idx, row) in wrap_text(text, text_width).into_iter().enumerate() {
        let lead = if idx == 0 {
            prefix.to_string()
        } else {
            " ".repeat(prefix.len())
        };
        lines.push(Line::from(Span::styled(format!("{lead}{row}"), style)));
    }
}

fn make_meter(score: u8, width: usize) -> String {
    let clamped = score.min(10) as usize;
    let filled = (clamped * width + 9) / 10;
    format!("{}{}", "#".repeat(filled), "-".repeat(width.saturating_sub(filled)))
}

fn star_line(width: usize, frame: u64, row: u64) -> String {
    let mut chars = vec![' '; width];
    let step = 7 + (row as usize % 3) * 3;
    for i in (0..width).step_by(step) {
        let p = (i + (frame as usize * (row as usize + 1))) % width;
        chars[p] = if (frame + row + i as u64) % 3 == 0 { '*' } else { '.' };
    }
    chars.into_iter().collect()
}

fn marquee(width: usize, frame: u64, text: &str) -> String {
    if width == 0 {
        return String::new();
    }

    let mut track = String::from("   ");
    track.push_str(text);
    track.push_str("   ");
    track.push_str(text);
    track.push_str("   ");

    let chars: Vec<char> = track.chars().collect();
    let len = chars.len();
    let start = (frame as usize) % len;
    (0..width).map(|i| chars[(start + i) % len]).collect()
}

fn runner_line(width: usize, frame: u64) -> (String, String) {
    if width < 16 {
        return ("M>".to_string(), "=".repeat(width));
    }

    let runner = if frame % 2 == 0 { "[M>]" } else { "[M^]" };
    let pos = (frame as usize) % (width - runner.len());

    let mut top = vec![' '; width];
    let mut base = vec!['='; width];

    for (idx, ch) in runner.chars().enumerate() {
        top[pos + idx] = ch;
    }

    for i in (4..width).step_by(9) {
        let c = (i + (frame as usize % 4)) % width;
        if top[c] == ' ' {
            top[c] = if ((frame / 2) + i as u64) % 2 == 0 { 'o' } else { '+' };
        }
    }

    if width > 8 {
        base[width - 8] = '|';
        base[width - 7] = '|';
        base[width - 6] = '|';
    }

    (top.into_iter().collect(), base.into_iter().collect())
}

pub fn build_resume_lines(area: Rect, frame: u64) -> Vec<Line<'static>> {
    let width = area.width.max(40) as usize;

    let title = Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD);
    let accent = Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD);
    let section = Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD);
    let text = Style::default().fg(Color::White);
    let dim = Style::default().fg(Color::Gray);
    let sky = Style::default().fg(Color::LightBlue);
    let link = Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let divider = "-".repeat(width);

    let glow = if frame % 4 < 2 { "LIVE" } else { "PIXEL" };
    lines.push(Line::from(Span::styled(star_line(width, frame, 1), sky)));
    lines.push(Line::from(Span::styled(star_line(width, frame, 2), sky)));
    lines.push(Line::from(vec![
        Span::styled(" PORTFOLIO WORLD :: ", accent),
        Span::styled(glow, title),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" Creator: ", dim),
        Span::styled("Riyan Khan", title),
        Span::styled(" | Role: ", dim),
        Span::styled("Software Engineer (Backend)", text),
    ]));

    let tape = marquee(width, frame, "JAVA  NODE  RUST  SYSTEM DESIGN  CLOUD  MICROSERVICES");
    lines.push(Line::from(Span::styled(tape, dim)));

    let (runner, platform) = runner_line(width, frame);
    lines.push(Line::from(Span::styled(runner, accent)));
    lines.push(Line::from(Span::styled(platform, Style::default().fg(Color::Green))));

    lines.push(Line::from(Span::styled(divider.clone(), dim)));
    lines.push(Line::from(vec![
        Span::styled("Controls: ", dim),
        Span::styled("[j/k] or [up/down] scroll", text),
        Span::styled("  [g/G] jump", text),
        Span::styled("  [q] quit", accent),
    ]));
    lines.push(Line::from(Span::styled(divider.clone(), dim)));

    lines.push(Line::from(Span::styled("ABOUT ME", section)));
    push_wrapped(
        &mut lines,
        "  ",
        "I build backend systems that are fast, reliable, and easy to evolve. Over 3+ years I have shipped APIs, realtime services, and fintech/blockchain integrations used in production.",
        text,
        width,
    );

    lines.push(Line::from(Span::styled(divider.clone(), dim)));
    lines.push(Line::from(Span::styled("WHAT I BRING", section)));
    let impact = [
        "Production backend engineering across Java, Node.js, and Rust.",
        "Strong API design and service architecture for scalable systems.",
        "CI/CD, containerized deployments, and cloud-native delivery workflows.",
        "Ownership mindset: from design discussion to stable production rollouts.",
    ];
    for point in impact {
        push_wrapped(&mut lines, "  + ", point, text, width);
    }

    lines.push(Line::from(Span::styled(divider.clone(), dim)));
    lines.push(Line::from(Span::styled("SKILL LOADOUT", section)));
    let meter_w = width.saturating_sub(34).clamp(10, 28);
    let loadout = [
        ("Java / Spring", 9_u8, "Spring Boot, WebFlux, clean service boundaries"),
        ("Node / Nest", 9_u8, "Realtime systems, integration-heavy backends"),
        ("Cloud / DevOps", 9_u8, "Docker, Kubernetes, AWS, GitHub Actions"),
        ("Data", 8_u8, "MySQL, PostgreSQL, schema and query optimization"),
    ];
    for (name, score, summary) in loadout {
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<14} ", name), dim),
            Span::styled(make_meter(score, meter_w), accent),
            Span::styled(format!("  {score}/10"), title),
        ]));
        push_wrapped(&mut lines, "    ", summary, text, width);
    }

    lines.push(Line::from(Span::styled(divider.clone(), dim)));
    lines.push(Line::from(Span::styled("FEATURED BUILDS", section)));
    let highlights = [
        (
            "Kryptoria",
            "Backend platform for wallet and NFT interactions using Spring Boot + Node.js.",
        ),
        (
            "Wrktalk",
            "Realtime messaging engine with reliable sync and low-latency communication.",
        ),
        (
            "Abra-Fi / DeFi",
            "Solana-integrated backend and payment rails for production fintech use cases.",
        ),
    ];
    for (project, detail) in highlights {
        lines.push(Line::from(vec![
            Span::styled("  > ", accent),
            Span::styled(project, title),
        ]));
        push_wrapped(&mut lines, "    ", detail, text, width);
    }

    lines.push(Line::from(Span::styled(divider.clone(), dim)));
    lines.push(Line::from(Span::styled("CONNECT", section)));
    lines.push(Line::from(vec![
        Span::styled("  Email: ", dim),
        Span::styled("riyankhanpyrex01@gmail.com", link),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  GitHub: ", dim),
        Span::styled("github.com/Pyrex01", link),
        Span::styled(" | LinkedIn: ", dim),
        Span::styled("riyan--khan", link),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Location: ", dim),
        Span::styled("Mumbai, India", text),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Open to: ", dim),
        Span::styled("Backend / Platform Engineering roles", title),
    ]));

    lines.push(Line::from(Span::styled(divider, dim)));
    lines.push(Line::from(vec![
        Span::styled(" THANKS FOR PLAYING ", accent),
        Span::styled("Built with Rust + ratatui.", text),
    ]));

    lines
}

pub fn render_resume_ui(buffer: &mut Buffer, area: Rect, scroll_offset: u16, frame: u64) -> usize {
    let lines = build_resume_lines(area, frame);
    let total_lines = lines.len();

    let start_y = scroll_offset as usize;
    let max_lines = area.height as usize;

    for (idx, line) in lines.iter().enumerate() {
        let y_pos = idx.saturating_sub(start_y);
        if y_pos < max_lines && idx >= start_y {
            Paragraph::new(line.clone()).render(Rect::new(0, y_pos as u16, area.width, 1), buffer);
        }
    }

    total_lines
}
