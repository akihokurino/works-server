use std::cmp;

const DEFAULT_LIMIT: i64 = 20;

pub struct Pager {
    page: i64,
    limit: i64,
}

impl Pager {
    pub fn new(page: i32, limit: i32) -> Pager {
        let _page = cmp::max(page, 1) as i64;
        let _limit = if limit > 0 {
            limit as i64
        } else {
            DEFAULT_LIMIT
        };

        Pager {
            page: _page,
            limit: _limit,
        }
    }

    pub fn get_offset(&self) -> i64 {
        self.page * self.limit - self.limit
    }

    pub fn get_limit(&self) -> i64 {
        self.limit
    }
}
