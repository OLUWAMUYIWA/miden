use super::{BTreeMap, Felt, StarkField, Word, INIT_MEM_VALUE};

// MEMORY SEGMENT TRACE
// ================================================================================================

/// Memory access trace for a single sorted first by address and then by clock cycle.
#[derive(Default)]
pub struct MemorySegment(BTreeMap<u64, Vec<(Felt, Word)>>);

impl MemorySegment {
    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns a word located at the specified address, or None if the address hasn't been
    /// accessed previously.
    ///
    /// Unlike read() that modifies the underlying map, get_value() only attempts to read
    /// or return None when no value exists.
    pub fn get_value(&self, addr: u64) -> Option<Word> {
        match self.0.get(&addr) {
            Some(addr_trace) => addr_trace.last().map(|(_, value)| *value),
            None => None,
        }
    }

    /// Returns the entire memory state at the beginning of the specified cycle.
    pub fn get_state_at(&self, clk: u32) -> Vec<(u64, Word)> {
        let mut result: Vec<(u64, Word)> = Vec::new();

        if clk == 0 {
            return result;
        }

        // Because we want to view the memory state at the beginning of the specified cycle, we
        // view the memory state at the previous cycle, as the current memory state is at the
        // end of the current cycle.
        let search_clk = (clk - 1) as u64;

        for (&addr, addr_trace) in self.0.iter() {
            match addr_trace.binary_search_by(|(x, _)| x.as_int().cmp(&search_clk)) {
                Ok(i) => result.push((addr, addr_trace[i].1)),
                Err(i) => {
                    // Binary search finds the index of the data with the specified clock cycle.
                    // Decrement the index to get the trace from the previously accessed clock
                    // cycle to insert into the results.
                    if i > 0 {
                        result.push((addr, addr_trace[i - 1].1));
                    }
                }
            }
        }

        result
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    pub fn inner(&self) -> &BTreeMap<u64, Vec<(Felt, Word)>> {
        &self.0
    }

    pub fn into_inner(self) -> BTreeMap<u64, Vec<(Felt, Word)>> {
        self.0
    }

    // STATE MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Returns a word (4 elements) located in memory at the specified address.
    ///
    /// If the specified address hasn't been previously written to, four ZERO elements are
    /// returned. This effectively implies that memory is initialized to ZERO.
    pub fn read(&mut self, addr: Felt, clk: Felt) -> Word {
        // look up the previous value in the appropriate address trace and add (clk, prev_value)
        // to it; if this is the first time we access this address, create address trace for it
        // with entry (clk, [ZERO, 4]). in both cases, return the last value in the address trace.
        self.0
            .entry(addr.as_int())
            .and_modify(|addr_trace| {
                let last_value = addr_trace.last().expect("empty address trace").1;
                addr_trace.push((clk, last_value));
            })
            .or_insert_with(|| vec![(clk, INIT_MEM_VALUE)])
            .last()
            .expect("empty address trace")
            .1
    }

    /// Writes the provided word (4 elements) at the specified address.
    pub fn write(&mut self, addr: Felt, clk: Felt, value: Word) {
        // add a tuple (clk, value) to the appropriate address trace; if this is the first time
        // we access this address, initialize address trace.
        self.0
            .entry(addr.as_int())
            .and_modify(|addr_trace| addr_trace.push((clk, value)))
            .or_insert_with(|| vec![(clk, value)]);
    }

    // HELPER FUNCTIONS
    // --------------------------------------------------------------------------------------------

    #[cfg(test)]
    pub fn size(&self) -> usize {
        self.0.len()
    }
}
