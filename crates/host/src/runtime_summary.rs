//! Exact-field, payload-free summaries of existing kernel status records.
//! This reads no game image or instructions and adds no guest instrumentation.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub exceptions: usize,
    pub zero_pc: usize,
    pub nonzero_pc: usize,
    pub unavailable_pc: usize,
    pub zero_fault: usize,
    pub null_adjacent_fault: usize,
    pub vm_success: usize,
    pub vm_failure: usize,
}

fn hex_field(line: &str, key: &str) -> Option<u32> {
    let mut values = line.split_ascii_whitespace().filter_map(|word| {
        let (name, value) = word.split_once('=')?;
        (name == key).then_some(value)
    });
    let value = values.next()?;
    if values.next().is_some() || value.len() != 8 {
        return None;
    }
    u32::from_str_radix(value, 16).ok()
}

impl Summary {
    pub fn parse(text: &str) -> Self {
        let mut s = Self::default();
        for line in text.lines() {
            if line.starts_with("XBX-EXCEPTION ") {
                s.exceptions += 1;
                match hex_field(line, "pc") {
                    Some(0) => s.zero_pc += 1,
                    Some(_) => s.nonzero_pc += 1,
                    None => s.unavailable_pc += 1,
                }
                match hex_field(line, "fault") {
                    Some(0) => s.zero_fault += 1,
                    Some(1..=4095) => s.null_adjacent_fault += 1,
                    _ => (),
                }
            } else if line.starts_with("XBX-VM ") {
                match hex_field(line, "status") {
                    Some(v) if v & 0x80000000 == 0 => s.vm_success += 1,
                    Some(_) => s.vm_failure += 1,
                    None => (),
                }
            }
        }
        s
    }

    pub fn receipt(&self) -> String {
        format!(
            "schema=xbx-runtime-summary-v1\nexceptions={}\nzero_pc={}\nnonzero_pc={}\nunavailable_pc={}\nzero_fault={}\nnull_adjacent_fault={}\nrecorded_vm_success={}\nrecorded_vm_failure={}\nrecords_are_bounded_not_complete_history=true\ngameplay_unqualified=true\n",
            self.exceptions,
            self.zero_pc,
            self.nonzero_pc,
            self.unavailable_pc,
            self.zero_fault,
            self.null_adjacent_fault,
            self.vm_success,
            self.vm_failure
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_pc_is_independent_of_zero_stack_pc_and_fault() {
        let text = "XBX-EXCEPTION code=c0000005 pc=00012345 fault=00000004 origin=14 stack_pc=00000000\nXBX-VM status=00000000\nXBX-VM status=c0000018\n";
        assert_eq!(
            Summary::parse(text),
            Summary {
                exceptions: 1,
                nonzero_pc: 1,
                null_adjacent_fault: 1,
                vm_success: 1,
                vm_failure: 1,
                ..Summary::default()
            }
        );
        assert_eq!(
            Summary::parse("XBX-EXCEPTION pc=00000000 fault=00000000 stack_pc=00012345").zero_pc,
            1
        );
        assert_eq!(
            Summary::parse("XBX-EXCEPTION stack_pc=00000000").unavailable_pc,
            1
        );
        assert_eq!(
            Summary::parse("XBX-EXCEPTION pc=00012345 pc=00000000").unavailable_pc,
            1
        );
    }
}
