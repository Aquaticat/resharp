use crate::{accel::FwdPrefixSearch, engine, Error, Match, Regex};

impl Regex {
    pub(crate) fn find_all_fwd_prefix(&self, fwd_prefix: &FwdPrefixSearch, input: &[u8]) -> Result<Vec<Match>, Error> {
        let inner = &mut *self.inner.lock().unwrap();
        let matches = &mut inner.matches;
        matches.clear();
        let mut search_start = 0;

        if self.fixed_length == Some(fwd_prefix.len() as u32)
            && !self.has_anchors
            && fwd_prefix.find_all_literal(input, matches)
        {
        } else {
            {
                let mt = inner.fwd.mt_lookup[input[0] as usize];
                let state = inner.fwd.begin_table[mt as usize] as u32;
                if state != inner.fwd.pruned as u32 {
                    let max_end = inner.fwd.scan_fwd_from(&mut inner.b, state, 1, input)?;
                    if max_end != engine::NO_MATCH && max_end > 0 {
                        matches.push(Match {
                            start: 0,
                            end: max_end,
                        });
                        search_start = max_end;
                    }
                }
            }
            let prefix_len = fwd_prefix.len();
            while let Some(candidate) = fwd_prefix.find_fwd(input, search_start) {
                let state = inner
                    .fwd
                    .walk_input(&mut inner.b, candidate, prefix_len, input)?;
                if state != 0 {
                    let max_end = inner.fwd.scan_fwd_from(
                        &mut inner.b,
                        state,
                        candidate + prefix_len,
                        input,
                    )?;
                    if max_end != engine::NO_MATCH && max_end > candidate {
                        matches.push(Match {
                            start: candidate,
                            end: max_end,
                        });
                        search_start = max_end;
                        continue;
                    }
                }
                search_start = candidate + 1;
            }
        }

        Ok(matches.clone())
    }

    pub(crate) fn is_match_fwd_prefix(&self, fwd_prefix: &FwdPrefixSearch, input: &[u8]) -> Result<bool, Error> {
        let inner = &mut *self.inner.lock().unwrap();
        let prefix_len = fwd_prefix.len();
        {
            let mt = inner.fwd.mt_lookup[input[0] as usize];
            let state = inner.fwd.begin_table[mt as usize] as u32;
            if state != inner.fwd.pruned as u32 {
                let max_end = inner.fwd.scan_fwd_from(&mut inner.b, state, 1, input)?;
                if max_end != engine::NO_MATCH && max_end > 0 {
                    return Ok(true);
                }
            }
        }
        let mut search_start = 0;
        while let Some(candidate) = fwd_prefix.find_fwd(input, search_start) {
            let state = inner
                .fwd
                .walk_input(&mut inner.b, candidate, prefix_len, input)?;
            if state != 0 {
                let max_end = inner.fwd.scan_fwd_from(
                    &mut inner.b,
                    state,
                    candidate + prefix_len,
                    input,
                )?;
                if max_end != engine::NO_MATCH && max_end > candidate {
                    return Ok(true);
                }
            }
            search_start = candidate + 1;
        }
        Ok(false)
    }

    pub(crate) fn is_match_fwd_lb_prefix(&self, fwd_prefix: &FwdPrefixSearch, input: &[u8]) -> Result<bool, Error> {
        let inner = &mut *self.inner.lock().unwrap();
        let lb_len = self.lb_check_bytes as usize;
        if self.fwd_lb_begin_nullable && !input.is_empty() {
            let max_end = inner.fwd.scan_fwd_slow(&mut inner.b, 0, input)?;
            if max_end != engine::NO_MATCH {
                return Ok(true);
            }
        }
        let mut search_start = 0usize;
        while let Some(candidate) = fwd_prefix.find_fwd(input, search_start) {
            let body_start = candidate + lb_len;
            let max_end = inner.fwd.scan_fwd_from(
                &mut inner.b,
                engine::DFA_INITIAL as u32,
                body_start,
                input,
            )?;
            if max_end != engine::NO_MATCH {
                return Ok(true);
            }
            search_start = body_start;
        }
        Ok(false)
    }

    pub(crate) fn find_all_fwd_lb_prefix(&self, fwd_prefix: &FwdPrefixSearch, input: &[u8]) -> Result<Vec<Match>, Error> {
        let inner = &mut *self.inner.lock().unwrap();
        inner.matches.clear();
        let lb_len = self.lb_check_bytes as usize;
        let mut search_start = 0usize;

        if self.fwd_lb_begin_nullable && !input.is_empty() {
            let max_end = inner.fwd.scan_fwd_slow(&mut inner.b, 0, input)?;
            if max_end != engine::NO_MATCH {
                inner.matches.push(Match {
                    start: 0,
                    end: max_end,
                });
                search_start = if max_end == 0 { 1 } else { max_end };
            }
        }

        while let Some(candidate) = fwd_prefix.find_fwd(input, search_start) {
            let body_start = candidate + lb_len;
            let max_end = inner.fwd.scan_fwd_from(
                &mut inner.b,
                engine::DFA_INITIAL as u32,
                body_start,
                input,
            )?;
            if max_end != engine::NO_MATCH {
                inner.matches.push(Match {
                    start: body_start,
                    end: max_end,
                });
                search_start = max_end;
            } else {
                search_start = body_start;
            }
        }

        Ok(inner.matches.clone())
    }
}
