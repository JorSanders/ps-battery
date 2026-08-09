/// Fills `buffer` with `text` as NUL terminated UTF-16, truncating the text
/// when it does not fit. Leftover bytes from a previous longer text may remain
/// past the terminator; Windows stops reading at the NUL.
pub fn copy_str_to_utf16_buffer(text: &str, buffer: &mut [u16]) {
    if buffer.is_empty() {
        return;
    }
    let text_utf16: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    let copy_len = text_utf16.len().min(buffer.len());
    buffer[..copy_len].copy_from_slice(&text_utf16[..copy_len]);
    if copy_len == buffer.len() {
        buffer[copy_len - 1] = 0;
    }
}
