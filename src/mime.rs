// TODO: Barely finished
// May refer here for mimetypes
// https://github.com/PapirusDevelopmentTeam/papirus-icon-theme/tree/master/Papirus/24x24/mimetypes

#[allow(dead_code)]
pub fn get_mimetypes(extension: &str) -> &'static str {
    match extension {
        "apk" => "application-vnd.android.package-archive",
        "blend" => "application-x-blender",
        "dart" => "application-dart",
        "mp3" | "ogg" | "wav" => "audio-x-generic",
        "mkv" | "mp4" => "video-x-generic",
        "rs" => "text-x-rust",
        _ => "text-x-generic",
    }
}
