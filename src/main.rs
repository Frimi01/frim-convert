use eframe::egui;
use rfd::FileDialog;
use std::path::Path;
use std::process::Command;
use std::str;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "FriConvert",
        options,
        Box::new(|_cc| Box::new(TrimApp::default())),
    )
}

#[derive(Default)]
struct TrimApp {
    input_path: String,
    output_path: String,
    start_time: String,
    end_time: String,
    video_length: String,
    is_video: bool,
}

impl eframe::App for TrimApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Input File Selection
            if ui.button("Select Input File").clicked() {
                if let Some(path) = FileDialog::new().pick_file() {
                    self.input_path = path.display().to_string();
                    self.set_default_output(); // Set default output path
                }
            }
            ui.label("Input File:");
            ui.text_edit_singleline(&mut self.input_path);

            ui.label("Output File:");
            ui.text_edit_singleline(&mut self.output_path);

            ui.checkbox(&mut self.is_video, "Trim video file");

            if self.is_video == true {
                ui.label("Start Time (HH:MM:SS or seconds):");
                ui.text_edit_singleline(&mut self.start_time);

                ui.label("End Time (HH:MM:SS or seconds):");
                ui.text_edit_singleline(&mut self.end_time);
            }

            if ui.button("Convert and Modify").clicked() {
                self.convert();
            }
        });
    }
}

impl TrimApp {
    fn set_default_output(&mut self) {
        if !self.input_path.is_empty() {
            if let Some(parent) = Path::new(&self.input_path).parent() {
                let output_filename = "trimmed_output.mp4";
                self.output_path = parent.join(output_filename).display().to_string();
            }
            self.video_length = self.get_video_duration();
        }
    }

    fn get_video_duration(&self) -> String {
        if self.input_path.is_empty() {
            return "0".to_string();
        }

        let output = Command::new("ffmpeg")
            .args(["-i", &self.input_path, "-hide_banner", "-f", "null", "-"])
            .output();

        if let Ok(out) = output {
            let stderr = str::from_utf8(&out.stderr).unwrap_or("");
            if let Some(duration_line) = stderr.lines().find(|line| line.contains("Duration:")) {
                let parts: Vec<&str> = duration_line.split_whitespace().collect();
                if parts.len() > 1 {
                    return parts[1].trim_matches(',').to_string();
                }
            }
        }

        "0".to_string() // Fallback if extraction fails
    }

    fn convert(&self) {
        if self.input_path.is_empty() || self.output_path.is_empty() {
            println!("Error: Input and output paths cannot be empty.");
            return;
        }
        if self.is_video == true {
            self.convert_video();
        } else {
            self.convert_image();
        }
    }

    fn convert_video(&self) {
        let start = if self.start_time.is_empty() {
            "0".to_string()
        } else {
            self.start_time.clone()
        };
        let end = if self.end_time.is_empty() {
            self.video_length.clone()
        } else {
            self.end_time.clone()
        };

        let ffmpeg_cmd = Command::new("ffmpeg")
            .args([
                "-i",
                &self.input_path,
                "-ss",
                &start,
                "-to",
                &end,
                "-c",
                "copy",
                &self.output_path,
            ])
            .output();

        match ffmpeg_cmd {
            Ok(output) => {
                if output.status.success() {
                    println!("Conversion successful!");
                } else {
                    println!("Error: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => println!("Failed to execute FFmpeg: {}", e),
        }
    }

    fn convert_image(&self) {
        let ffmpeg_cmd = Command::new("ffmpeg")
            .args(["-i", &self.input_path, &self.output_path])
            .output();

        match ffmpeg_cmd {
            Ok(output) => {
                if output.status.success() {
                    println!("Trim successful!");
                } else {
                    println!("Error: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => println!("Failed to execute FFmpeg: {}", e),
        }
    }
}
