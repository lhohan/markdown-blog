## Development Style Principles

- Start new development with a 'primitive whole' that is tested end-to-end. Ensure the program remains functional at all times.
- Write tests at the appropriate level, emphasizing test stability by focusing on behavior rather than implementation details.
- Write tests in a TDD style, focusing on the smallest change possible to achieve goals, even if it means starting with hardcoded values.
- When refactoring, clearly indicate the intention with "REFACTOR" comments, focusing on production code while assuming tests cover existing functionality.
