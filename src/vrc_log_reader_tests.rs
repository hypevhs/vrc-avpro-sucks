#[cfg(test)]
mod get_url_regex {
    #[test]
    fn it_extracts_example_a() {
        let url_regex = &crate::vrc_log_reader::URL_REGEX;
        let example_a = "2024.06.06 17:22:14 Log        -  [AT INFO    	TVManager (Theatre 1 TVManager)] [VideoManager_Theatre1] (Some Username) Now Playing: https://example.net/video.mp4";
        let example_captures = url_regex.captures(example_a).unwrap();
        assert_eq!(
            example_captures.name("timestamp").unwrap().as_str(),
            "2024.06.06 17:22:14"
        );
        assert_eq!(
            example_captures.name("player_name").unwrap().as_str(),
            "Theatre 1 TVManager"
        );
        assert_eq!(
            example_captures.name("url").unwrap().as_str(),
            "https://example.net/video.mp4"
        );
    }

    #[test]
    fn it_extracts_example_b() {
        let url_regex = &crate::vrc_log_reader::URL_REGEX;
        let example_b = "2024.07.22 17:48:43 Log        -  [ATA | TVManagerV2 (ProTV Hangout (1))] [AVPro1080p] Now Playing: https://youtu.be/zL3wWykAKfs";
        let example_captures = url_regex.captures(example_b).unwrap();
        assert_eq!(
            example_captures.name("timestamp").unwrap().as_str(),
            "2024.07.22 17:48:43"
        );
        assert_eq!(
            example_captures.name("player_name").unwrap().as_str(),
            "ProTV Hangout (1)"
        );
        assert_eq!(
            example_captures.name("url").unwrap().as_str(),
            "https://youtu.be/zL3wWykAKfs"
        );
    }
}

#[cfg(test)]
mod get_seek_regex {
    #[test]
    fn it_extracts_example_a() {
        let seek_regex = &crate::vrc_log_reader::SEEK_REGEX;
        let example_a = "2024.04.22 17:55:53 Log        -  [AT INFO   TVManager (Theatre 1 TVManager)] Sync enforcement. Updating to 116.47";
        let example_captures = seek_regex.captures(example_a).unwrap();
        assert_eq!(
            example_captures.name("timestamp").unwrap().as_str(),
            "2024.04.22 17:55:53"
        );
        assert_eq!(
            example_captures.name("player_name").unwrap().as_str(),
            "Theatre 1 TVManager"
        );
        assert_eq!(
            example_captures.name("new_offset").unwrap().as_str(),
            "116.47"
        );
    }

    #[test]
    fn it_extracts_example_b() {
        let seek_regex = &crate::vrc_log_reader::SEEK_REGEX;
        let example_b = "2024.05.09 19:11:19 Log        -  [AT DEBUG  TVManager (Theatre 1 TVManager)] Paused drift threshold exceeded. Updating to 64.8041";
        let example_captures = seek_regex.captures(example_b).unwrap();
        assert_eq!(
            example_captures.name("timestamp").unwrap().as_str(),
            "2024.05.09 19:11:19"
        );
        assert_eq!(
            example_captures.name("player_name").unwrap().as_str(),
            "Theatre 1 TVManager"
        );
        assert_eq!(
            example_captures.name("new_offset").unwrap().as_str(),
            "64.8041"
        );
    }

    #[test]
    fn it_extracts_example_c() {
        let seek_regex = &crate::vrc_log_reader::SEEK_REGEX;
        let example_c = "2024.06.03 18:03:02 Log        -  [AT DEBUG  TVManager (Theatre 3 TVManager)] Jumping [VideoManager_Theatre3] to timestamp: 171.1321";
        let example_captures = seek_regex.captures(example_c).unwrap();
        assert_eq!(
            example_captures.name("timestamp").unwrap().as_str(),
            "2024.06.03 18:03:02"
        );
        assert_eq!(
            example_captures.name("player_name").unwrap().as_str(),
            "Theatre 3 TVManager"
        );
        assert_eq!(
            example_captures.name("new_offset").unwrap().as_str(),
            "171.1321"
        );
    }

    #[test]
    fn it_extracts_example_d() {
        let seek_regex = &crate::vrc_log_reader::SEEK_REGEX;
        let example_d = "2024.07.22 17:48:44 Log        -  [ATA | TVManagerV2 (ProTV Hangout (1))] Sync enforcement requested. Updating to 44.96499";
        let example_captures = seek_regex.captures(&example_d).unwrap();
        assert_eq!(
            example_captures.name("timestamp").unwrap().as_str(),
            "2024.07.22 17:48:44"
        );
        assert_eq!(
            example_captures.name("player_name").unwrap().as_str(),
            "ProTV Hangout (1)"
        );
        assert_eq!(
            example_captures.name("new_offset").unwrap().as_str(),
            "44.96499"
        );
    }
}
