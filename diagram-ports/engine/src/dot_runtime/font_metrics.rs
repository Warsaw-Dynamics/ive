// SPDX-License-Identifier: EPL-2.0
// ASCII advance data adapted from Graphviz 16.1.0 lib/common/textspan_lut.c.
// Copyright (c) Graphviz contributors. See vendor/graphviz/COPYING and
// THIRD-PARTY-NOTICES.md.

const TIMES_ROMAN_ASCII: [u16; 95] = [
    512, 682, 836, 1024, 1024, 1706, 1593, 369, 682, 682, 1024, 1155, 512, 682, 512, 569, 1024,
    1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 1024, 569, 569, 1155, 1155, 1155, 909, 1886,
    1479, 1366, 1366, 1479, 1251, 1139, 1479, 1479, 682, 797, 1479, 1251, 1821, 1479, 1479, 1139,
    1479, 1366, 1139, 1251, 1479, 1479, 1933, 1479, 1479, 1251, 682, 569, 682, 961, 1024, 682, 909,
    1024, 909, 1024, 909, 682, 1024, 1024, 569, 569, 1024, 569, 1593, 1024, 1024, 1024, 1024, 682,
    797, 569, 1024, 1024, 1479, 1024, 1024, 909, 983, 410, 983, 1108,
];

pub(super) fn ascii_advance(text: &str, font: &str, size: f64) -> Option<f64> {
    if !text.bytes().all(|byte| (32..=126).contains(&byte)) {
        return None;
    }
    let units = match font {
        "Times-Roman" => text
            .bytes()
            .map(|byte| u64::from(TIMES_ROMAN_ASCII[usize::from(byte - 32)]))
            .sum(),
        "Courier" => (text.len() as u64).checked_mul(1229)?,
        _ => return None,
    };
    Some(units as f64 / 2048. * size)
}

#[cfg(test)]
mod tests {
    use super::ascii_advance;

    #[test]
    fn should_match_graphviz_161_ascii_advance_data() {
        // given
        let times = "quoted id";
        let courier = " ";
        // when
        let times_width = ascii_advance(times, "Times-Roman", 14.).unwrap();
        let courier_width = ascii_advance(courier, "Courier", 12.).unwrap();
        // then
        assert_eq!(times_width, 52.4931640625);
        assert_eq!(courier_width, 7.201171875);
    }
}
