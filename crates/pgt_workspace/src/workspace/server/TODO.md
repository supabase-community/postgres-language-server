1. Statement Iterator

```rust
pub struct StatementIterator<'a> {
    document: &'a Document,
    positions: std::slice::Iter<'a, StatementPos>,
    include_text: bool,
    include_range: bool,
}

impl<'a> StatementIterator<'a> {
    fn new(document: &'a Document) -> Self {
        Self {
            document,
            positions: document.positions.iter(),
            include_text: false,
            include_range: false,
        }
    }

    pub fn with_text(mut self) -> Self {
        self.include_text = true;
        self
    }

    pub fn with_range(mut self) -> Self {
        self.include_range = true;
        self
    }
}

pub enum StatementData<'a> {
    Statement(Statement),
    WithText(Statement, &'a str),
    WithRange(Statement, &'a TextRange),
    WithTextAndRange(Statement, &'a TextRange, &'a str),
}

impl<'a> Iterator for StatementIterator<'a> {
    type Item = StatementData<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.positions.next().map(|(id, range)| {
            let statement = Statement {
                id: *id,
                path: self.document.path.clone(),
            };

            match (self.include_text, self.include_range) {
                (false, false) => StatementData::Statement(statement),
                (true, false) => {
                    let text = &self.document.content[range.start().into()..range.end().into()];
                    StatementData::WithText(statement, text)
                },
                (false, true) => StatementData::WithRange(statement, range),
                (true, true) => {
                    let text = &self.document.content[range.start().into()..range.end().into()];
                    StatementData::WithTextAndRange(statement, range, text)
                },
            }
        })
    }
}
pub struct StatementIterator<'a> {
    document: &'a Document,
    positions: std::slice::Iter<'a, StatementPos>,
    include_text: bool,
    include_range: bool,
}

impl<'a> StatementIterator<'a> {
    fn new(document: &'a Document) -> Self {
        Self {
            document,
            positions: document.positions.iter(),
            include_text: false,
            include_range: false,
        }
    }

    pub fn with_text(mut self) -> Self {
        self.include_text = true;
        self
    }

    pub fn with_range(mut self) -> Self {
        self.include_range = true;
        self
    }
}

pub enum StatementData<'a> {
    Statement(Statement),
    WithText(Statement, &'a str),
    WithRange(Statement, &'a TextRange),
    WithTextAndRange(Statement, &'a TextRange, &'a str),
    // with ast
    // with cst
    // include substatements
}

impl<'a> Iterator for StatementIterator<'a> {
    type Item = StatementData<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.positions.next().map(|(id, range)| {
            let statement = Statement {
                id: *id,
                path: self.document.path.clone(),
            };

            match (self.include_text, self.include_range) {
                (false, false) => StatementData::Statement(statement),
                (true, false) => {
                    let text = &self.document.content[range.start().into()..range.end().into()];
                    StatementData::WithText(statement, text)
                },
                (false, true) => StatementData::WithRange(statement, range),
                (true, true) => {
                    let text = &self.document.content[range.start().into()..range.end().into()];
                    StatementData::WithTextAndRange(statement, range, text)
                },
            }
        })
    }
}
```

2. Parser
- one instance per document
- hold ts parser
- has "inner" document
- holds parse results
- exposes unified api to find statements with data
- reason for putting this together is that we dont want to manually fiddle with sub statements client side
-> i want to do doc.statements() and get all statements WITH sub statements.

