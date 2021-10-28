use std::ops::Range;

pub struct ChangeSet {
    pub filename: String,
    pub code: String,
    pub lines: Vec<usize>
}

impl ChangeSet {
    pub fn new(filename: &str, code: &str) -> ChangeSet {
        ChangeSet{
            filename: filename.into(), 
            code: code.into(), 
            lines: vec![]
        }
    }

    pub fn add_line(&mut self, line_number: usize) {
        self.lines.push(line_number);
    }

    pub fn ranges(&self) -> Vec<Range<usize>> {
        let mut ranges: Vec<Range<usize>> = vec![];    
        if self.lines.is_empty() {
            return ranges
        }

		let start = *self.lines.get(0).unwrap();

		let mut next = start;
		for end in self.lines.iter().skip(1) {
			if next + 1 != *end {
				ranges.push(start..next+1);
			}
			next = *end;
		}
		ranges.push(start..next+1);

        ranges
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_change_set_test() {
        let mut cs = ChangeSet::new("", "");
        let expected_line_number: usize = 3;
        cs.add_line(expected_line_number);
        assert!(cs.ranges().iter().find(|r| r.contains(&expected_line_number)).is_some());
    }

    #[test]
    fn contains_single_range_for_single_line(){
        let mut cs = ChangeSet::new("", "");
        let expected_line_number: usize = 3;
        cs.add_line(expected_line_number);
        assert_eq!(cs.ranges().len(), 1);
    }

    #[test]
    fn contains_single_range_for_two_consecutive_lines(){
        let mut cs = ChangeSet::new("", "");
        (1..3).for_each(|lino| cs.add_line(lino as usize));
        assert_eq!(cs.ranges().len(), 1);
    }

    #[test]
    fn contains_two_ranges_for_two_nonconsecutive_lines(){
        let mut cs = ChangeSet::new("", "");
        vec![1, 3].iter().for_each(|lino| cs.add_line(*lino as usize));
        assert_eq!(cs.ranges().len(), 2);
    }

    #[test]
    fn contains_single_range_for_three_consecutive_numbers_test() {
        let mut cs = ChangeSet::new("", "");
        (1..=3).for_each(|lino| cs.add_line(lino as usize));
        dbg!(cs.ranges());
        assert_eq!(cs.ranges().len(), 1);
        (1..=3).for_each(|lino| assert!(cs.ranges().iter().find(|r| r.contains(&lino)).is_some()))
    }

    #[test]
    fn contains_two_ranges_for_two_consecutive_numbers_and_another_test() {
        let mut cs = ChangeSet::new("", "");
        (1..3).for_each(|lino| cs.add_line(lino as usize));
        cs.add_line(4);
        dbg!(cs.ranges());
        assert_eq!(cs.ranges().len(), 2);
        vec![1, 2, 4].iter()
            .for_each(|lino| assert!(cs.ranges().iter().find(|r| r.contains(&lino)).is_some()))
    }

    #[test]
    fn contains_multiple_lines_test() {
        let mut cs = ChangeSet::new("", "");
        let expected_line_numbers: Vec<usize> = vec![1, 2, 3, 4];
        for v in &expected_line_numbers {
            cs.add_line(*v);
        }

        dbg!(cs.ranges());
        for v in &expected_line_numbers {
            assert!(cs.ranges().iter().find(|r| r.contains(v)).is_some());
        }
    }

    #[test]
    fn contains_only_single_range_for_consecutive_numbers() {
        let mut cs = ChangeSet::new("", "");
        (1..4).for_each(|v| cs.add_line(v as usize) );

        assert_eq!(cs.ranges().len(), 1);
    }

}
