use super::*;

/// A sized terminal text area.
// As a general hint to people writing implementations for [TextArea], it's ok to use the color and
// cursor fields directly, but to set them its better to use the setters since they check if
// everything is valid. Panic if its expected that it is valid -- it's better than leaving it in an
// invalid state.
pub struct TextArea<'w, 'b, E: Debug, T: TerminalOutput<E> + 'w> {
    writer: &'w Mutex<T>,
    // TODO min point max point wat
    bottom_left: Point,
    top_right: Point,
    resolution: Resolution,
    buffer: &'b mut [&'b mut [TerminalCharacter]],
    /// The area's cursor. Global -- origin is underlying writer's origin
    cursor: Point,
    color: ColorPair,
    _phantom: PhantomData<E>,
}

impl<'w, 'b, E: Debug, T: TerminalOutput<E> + 'w> TextArea<'w, 'b, E, T> {
    /// Creates a new TextArea with a writer Mutex for the terminal to write to and the given bounds
    ///
    /// # Note
    ///
    /// At the moment, it is required that the user passes in a buffer of the same size as the
    /// resolution. When const-generics is implemented in nightly, it will be able to create its own
    /// buffer.
    pub fn new(writer: &'w Mutex<T>, origin: Point, size: Resolution, buffer: &'b mut [&'b mut [TerminalCharacter]]) -> Self {
        TextArea {
            writer,
            bottom_left: origin,
            top_right: Point {
                x: origin.x + size.x,
                y: origin.y + size.y,
            },
            resolution: size,
            buffer,
            cursor: origin,
            color: color!(White on Black),
            _phantom: PhantomData,
        }
    }

    // TODO do these work? write test
    /// Preserves the state of the underlying writer, letting the area write to it with no fear of
    /// having to set it back again.
    ///
    /// # Side Effects
    ///
    /// This function locks the underlying writer to make sure that it isn't resized while it has an
    /// old state stored so that there isn't an error setting it back to the old state. Use the
    /// second argument of the closure to access the writer.
    ///
    /// # Closure arguments
    ///
    /// 1. The text area. Required to fix borrowing
    /// 2. The locked underlying writer. See side effects.
    ///
    /// # What It Does
    /// Sets the cursor pos and color of the underlying writer to the area's, executes the closure,
    /// and then sets the underlying writer's cursor and color and the area's cursor and color back
    /// to what it was
    fn with_state<F, R>(&mut self, f: F) -> Result<R, TerminalOutputError<E>>
        where F: FnOnce(&mut Self, &mut T) -> Result<R, TerminalOutputError<E>>
    {
        // Lock writer -- we don't want changes to it while we're working...
        // TODO -- is this right? Where else should we do this?
        let mut writer = self.writer.lock();

        // Store old state
        let old_writer_state = (writer.cursor_pos(), writer.color());
        let old_area_state = (self.cursor, self.color);

        // Sync state
        // Scope because temp variables
        {
            let cursor_pos = self.cursor_pos();
            writer.set_cursor_pos(cursor_pos)
                .expect("Cursor in bounds of TextArea should be in bounds of underlying writer");

            let color = self.color;
            writer.set_color(color)
                .expect("Color supported by TextArea should be supported by underlying writer");
        }

        let ret = f(self, &mut *writer);

        // Reset state
        writer.set_cursor_pos(old_writer_state.0)
            .expect("Old writer cursor position should still be in bounds");
        writer.set_color(old_writer_state.1).expect("Old writer color should still be supported");
        self.set_cursor_pos(old_area_state.0)
            .expect("Old area cursor position should still be in bounds");
        self.set_color(old_area_state.1).expect("Old area color should still be supported");

        ret
    }

    /// Preserves the state of the underlying writer, letting the area write to it with no fear of
    /// having to set it back again. Writes changes back to the area.
    ///
    /// # Side Effects
    ///
    /// This function locks the underlying writer to make sure that it isn't resized while it has an
    /// old state stored so that there isn't an error setting it back to the old state. Use the
    /// second argument of the closure to access the writer.
    ///
    /// # Closure arguments
    ///
    /// 1. The text area. Required to fix borrowing
    /// 2. The locked underlying writer. It's locked for internal reasons by the function
    ///
    /// # What It Does
    ///
    /// Sets the cursor pos and color of the underlying writer to the area's, executes the closure,
    /// sets the area's cursor and color to the writer's cursor and color, and then sets the
    /// underlying writer's position back to what it was.
    ///
    /// # Note
    ///
    // TODO: Rustioson
    fn with_state_writeback<F, R>(&mut self, f: F) -> Result<R, TerminalOutputError<E>>
        where F: FnOnce(&mut Self, &mut T) -> Result<R, TerminalOutputError<E>>
    {
        // Lock writer -- we don't want changes to it while we're working...
        let mut writer = self.writer.lock();

        // Store old state
        let old_writer_state = (writer.cursor_pos(), writer.color());

        // Sync state
        // Scope because temp variables
        {
            let cursor_pos = self.cursor_pos();
            writer.set_cursor_pos(cursor_pos)
                .expect("Cursor in bounds of TextArea should be in bounds of underlying writer");

            let color = self.color;
            writer.set_color(color)
                .expect("Color supported by TextArea should be supported by underlying writer");
        }

        let ret = f(self, &mut *writer);

        // Write back state
        self.set_cursor_pos(writer.cursor_pos())?;
        self.set_color(writer.color())
            .expect("Color supported by underlying writer should be supported by TextArea");

        // Reset state
        writer.set_cursor_pos(old_writer_state.0)
            .expect("Old writer cursor position should still be in bounds");
        writer.set_color(old_writer_state.1).expect("Old writer color should still be supported");

        ret
    }

    /// Repaints the text area according to its internal buffer. This will overwrite any accidental
    /// changes made to the buffer, e.g by setting characters inside the text area using the containing
    /// terminal
    #[allow(dead_code)] // Part of API
    pub fn repaint(&mut self) -> Result<(), TerminalOutputError<E>> {
        self.with_state(|area, writer| {
            for line in area.buffer.iter() {
                for character in line.iter() {
                    writer.write_raw(*character)?;
                }

                // TODO: FIX
                writer.set_cursor_pos(Point { x: area.bottom_left.x, y: area.cursor_pos().y + 1 })
                    .expect("Position inside repaint should be correct");
            }

            Ok(())
        })
    }
}

impl<'a, 'b, E: Debug, T: TerminalOutput<E> + 'a> TerminalOutput<E> for TextArea<'a, 'b, E, T> {
    fn color_supported(&self, color: Color) -> bool {
        self.writer.lock().color_supported(color)
    }

    fn resolution(&self) -> Resolution {
        self.resolution
    }

    fn in_bounds(&self, point: Point) -> bool {
        point.x >= self.bottom_left.x &&
            point.y >= self.bottom_left.y &&
            point.x <= self.top_right.x &&
            point.y <= self.top_right.y
    }

    fn set_cursor_pos(&mut self, point: Point) -> Result<(), TerminalOutputError<E>> {
        if self.in_bounds(point) {
            self.cursor = point;
            Ok(())
        } else {
            Err(TerminalOutputError::Debug(point, self.bottom_left, self.top_right))
        }
    }

    fn cursor_pos(&self) -> Point {
        self.cursor
    }

    fn color(&self) -> ColorPair {
        self.color
    }

    fn set_color(&mut self, color: ColorPair) -> Result<(), TerminalOutputError<E>> {
        if !self.color_supported(color.background) {
            return Err(TerminalOutputError::ColorUnsupported(color.background));
        }

        if !self.color_supported(color.foreground) {
            return Err(TerminalOutputError::ColorUnsupported(color.foreground));
        }

        self.color = color;
        Ok(())
    }

    fn set_char(&mut self, char: TerminalCharacter, point: Point) -> Result<(), TerminalOutputError<E>> {
        if self.in_bounds(point) {
            self.buffer[point.y][point.x] = char;
            self.writer.lock().set_char(char, point)
        } else {
            Err(TerminalOutputError::OutOfBounds(point))
        }
    }

    fn write_raw(&mut self, character: TerminalCharacter) -> Result<(), TerminalOutputError<E>> {
        self.with_state_writeback(|area, writer| {
            writer.write_raw(character)?;
            area.buffer[area.cursor.y][area.cursor.x] = character;

            // If cursor is out of bounds, wrap
            if !area.in_bounds(writer.cursor_pos()) {
                area.new_line()?;
                writer.set_cursor_pos(area.cursor_pos())?;
            }

            Ok(())
        })
    }

    fn write_string_colored(&mut self, str: &str, color: ColorPair) -> Result<(), TerminalOutputError<E>> {
        self.with_state_writeback(|area, writer| {
            for character in str.chars() {
                // We don't call TextArea.write_raw because writer is locked by with_state_writeback
                // already

                let terminal_character = TerminalCharacter { character, color };
                writer.write_colored(character, color)?;
                let local_pos = Point::new(
                    area.cursor.x - area.bottom_left.x,
                    area.cursor.y - area.bottom_left.y,
                );

                area.buffer[local_pos.y][local_pos.x] = terminal_character;

                // If cursor is out of bounds, wrap
                if !area.in_bounds(writer.cursor_pos()) {
                    area.new_line()?;
                    // TODO
//                    let bottom_left = area.bottom_left;
//                    area.set_cursor_pos(bottom_left)?;
                    writer.set_cursor_pos(area.cursor_pos())?;
                } else {
                    area.set_cursor_pos(writer.cursor_pos())
                        .expect("Point should be valid");
                }
            }

            Ok(())
        })
    }

    fn clear_line(&mut self, y: usize) -> Result<(), TerminalOutputError<E>> {
        self.with_state(|area, writer| {
            let x = area.bottom_left.x;
            area.set_cursor_pos(Point { x, y })?;

            for _ in area.bottom_left.x..area.top_right.x {
                writer.write(' ')?;
            }

            Ok(())
        })
    }

    fn clear(&mut self) -> Result<(), TerminalOutputError<E>> {
        for y in self.bottom_left.y..self.top_right.y {
            self.clear_line(y)?;
        }

        Ok(())
    }

    fn scroll_down(&mut self, lines: usize) -> Result<(), TerminalOutputError<E>> {
        let top_right = self.top_right;
        let bottom_left = self.bottom_left;
        self.clear_line(top_right.y)?;
        self.set_cursor_pos(Point::new(bottom_left.x, top_right.y)).expect("Min point should not be out of bounds");

        // Shift lines left (up) by amount only if amount < Y resolution
        // If amount is any more then the data will be cleared anyway
        if lines < self.resolution.y {
            self.buffer.rotate(lines);
        }

        for line in 0..lines {
            self.clear_line(line)?;
        }

        Ok(())
    }

    fn new_line(&mut self) -> Result<(), TerminalOutputError<E>> {
        let pos = self.cursor_pos();
        let new_x = self.bottom_left.x;
        if pos.y > self.bottom_left.y {
            self.set_cursor_pos(Point::new(new_x, pos.y - 1))?;
        } else {
            // TODO scroll
//            return Err(TerminalOutputError::BackspaceUnsupported);
        }
        Ok(())
    }

    fn backspace(&mut self) -> Result<(), TerminalOutputError<E>> {
        // TODO audit top left/bottom left
        // If backspace is possible
        if self.cursor.x > self.bottom_left.x || self.cursor.y > self.bottom_left.y {
            // If at start of line, move up a line, else move back in the current line
            if self.cursor.x == self.bottom_left.x {
                self.cursor.x = self.top_right.x - 1; // TODO use set_cursor_pos
                self.cursor.y -= 1; // TODO same
            } else {
                self.cursor.x -= 1; // TODO same
            }

            self.writer.lock().set_char(
                TerminalCharacter {
                    character: ' ',
                    color: self.color,
                },
                self.cursor,
            )?;

            Ok(())
        } else {
            Err(TerminalOutputError::BackspaceUnavailable(
                BackspaceUnavailableCause::TopOfTerminal
            ))
        }
    }
}
