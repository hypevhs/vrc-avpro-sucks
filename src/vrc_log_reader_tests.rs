#[cfg(test)]
mod get_url_regex {
    #[test]
    fn it_extracts_examples() {
        let url_regex = &crate::vrc_log_reader::URL_REGEX;
        let example_a = "2024.04.14 21:25:34 Log        -  [Video Playback] Attempting to resolve URL 'http://example.com/mystream/index.m3u8'";
        let example_b = "2024.04.15 18:03:42 Log        -  [Video Playback] Attempting to resolve URL 'https://example.net/animeclub_13.mp4'";
        let example_c = "2024.04.14 22:56:01 Log        -  [Video Playback] Attempting to resolve URL 'https://youtu.be/A4jc9RAUT6w'";
        let example_d = "2024.04.14 23:26:07 Log        -  [Video Playback] Attempting to resolve URL 'https://www.youtube.com/watch?v=TOjmh5ljiYs'";

        let example_a_captures = url_regex.captures(example_a).unwrap();
        assert_eq!(
            example_a_captures.get(1).unwrap().as_str(),
            "2024.04.14 21:25:34"
        );
        assert_eq!(
            example_a_captures.get(2).unwrap().as_str(),
            "http://example.com/mystream/index.m3u8"
        );

        let example_b_captures = url_regex.captures(example_b).unwrap();
        assert_eq!(
            example_b_captures.get(1).unwrap().as_str(),
            "2024.04.15 18:03:42"
        );
        assert_eq!(
            example_b_captures.get(2).unwrap().as_str(),
            "https://example.net/animeclub_13.mp4"
        );

        let example_c_captures = url_regex.captures(example_c).unwrap();
        assert_eq!(
            example_c_captures.get(1).unwrap().as_str(),
            "2024.04.14 22:56:01"
        );
        assert_eq!(
            example_c_captures.get(2).unwrap().as_str(),
            "https://youtu.be/A4jc9RAUT6w"
        );

        let example_d_captures = url_regex.captures(example_d).unwrap();
        assert_eq!(
            example_d_captures.get(1).unwrap().as_str(),
            "2024.04.14 23:26:07"
        );
        assert_eq!(
            example_d_captures.get(2).unwrap().as_str(),
            "https://www.youtube.com/watch?v=TOjmh5ljiYs"
        );
    }
}

#[cfg(test)]
mod get_seek_regex {
    #[test]
    fn it_extracts_example() {
        let seek_regex = &crate::vrc_log_reader::SEEK_REGEX;
        let example_a = "2024.04.22 17:55:53 Log        -  [AT INFO    	TVManager (Theatre 1 TVManager)] Sync enforcement. Updating to 116.47";
        let example_b = "2024.05.09 19:11:19 Log        -  [AT DEBUG 	TVManager (Theatre 1 TVManager)] Paused drift threshold exceeded. Updating to 64.8041";

        let example_captures = seek_regex.captures(example_a).unwrap();
        assert_eq!(
            example_captures.get(1).unwrap().as_str(),
            "2024.04.22 17:55:53"
        );
        assert_eq!(example_captures.get(4).unwrap().as_str(), "116.47");

        let example_captures = seek_regex.captures(example_b).unwrap();
        assert_eq!(
            example_captures.get(1).unwrap().as_str(),
            "2024.05.09 19:11:19"
        );
        assert_eq!(example_captures.get(4).unwrap().as_str(), "64.8041");
    }
}
