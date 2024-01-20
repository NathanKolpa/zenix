use core::fmt::{Display, Formatter};

use crate::util::{address::VirtualAddress, FixedVec};

use super::{MemoryMapper, NavigateCtx};

pub struct MemoryMapTreeDisplay<'a> {
    memory_mapper: &'a MemoryMapper,
    max_depth: u8,
}

impl<'a> MemoryMapTreeDisplay<'a> {
    pub fn new(memory_mapper: &'a MemoryMapper, max_depth: u8) -> Self {
        assert!(max_depth <= 3);
        Self {
            memory_mapper,
            max_depth,
        }
    }

    fn print_indentation_line(
        f: &mut Formatter<'_>,
        depth: u8,
        skip: &[bool],
    ) -> core::fmt::Result {
        for i in 0..depth {
            let char = if skip[i as usize] { " " } else { "│" };

            write!(f, "{char}   ")?;
        }

        Ok(())
    }
    fn print_present_indentation_line(
        f: &mut Formatter<'_>,
        depth: u8,
        skip: &[bool],
        last: bool,
    ) -> core::fmt::Result {
        Self::print_indentation_line(f, depth, skip)?;

        if last {
            write!(f, "└")?;
        } else {
            write!(f, "├")?;
        }

        write!(f, "───")
    }

    fn print_skipped_indentation_line(
        f: &mut Formatter<'_>,
        depth: u8,
        skip: &[bool],
        start: u16,
        end: u16,
    ) -> core::fmt::Result {
        Self::print_indentation_line(f, depth, skip)?;
        write!(f, "{start}..{end}")
    }
}

impl<'a> Display for MemoryMapTreeDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut skip_list = FixedVec::<4, _>::initialized_with(false);
        writeln!(f, "CR3")?;

        let mut skips = 0;
        let mut prev_depth = 0;

        let print_entry = |ctx: NavigateCtx| {
            let entry = ctx.entry;
            let entry_index = ctx.entry_index;

            if prev_depth != ctx.depth {
                if ctx.depth < prev_depth {
                    skip_list[ctx.depth as usize] = false;
                }

                prev_depth = ctx.depth;
                skips = 0;
            }

            if ctx.is_last_present_entry {
                skip_list[ctx.depth as usize] = true;
            }

            if !entry.flags().present() {
                skips += 1;
                return Ok(());
            }

            if skips == 1 {
                Self::print_indentation_line(f, ctx.depth, &skip_list)?;
                writeln!(f, "│")?;
            } else if skips > 0 {
                Self::print_skipped_indentation_line(
                    f,
                    ctx.depth,
                    &skip_list,
                    entry_index - skips,
                    entry_index - 1,
                )?;
                writeln!(f)?;
            }

            Self::print_present_indentation_line(
                f,
                ctx.depth,
                &skip_list,
                ctx.is_last_present_entry,
            )?;

            writeln!(f, "{entry_index}: [{entry:?}]")?;
            skips = 0;

            Ok(())
        };

        self.memory_mapper.navigate(
            VirtualAddress::null(),
            usize::MAX,
            Some(self.max_depth as usize),
            print_entry,
        )
    }
}
