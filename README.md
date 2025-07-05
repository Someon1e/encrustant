# Generic chess engine written in Rust

## Features:
- UCI
- Bit boards
- Legal move generator
- Stalemate and checkmate detection
- Repetition detection
- Profile-guided optimisation
- Pondering

### Search
- Iterative deepening
- Principal variation search
- Fail-soft alpha-beta pruning
- Quiescence search
- Transposition table
- Aspiration windows
- SPSA-tuned search parameters
- Check extensions
- `improving` heuristic

### Search pruning and reductions
- Late move reduction
- Late move pruning
- Null move heuristic
- Static null move pruning (also known as reverse futility pruning)
- Futility pruning
- Internal iterative reduction

### Evaluation
- Piece-square-table-only evaluation tuned on the lichess-big3-resolved dataset
- Pawn correction history
- Minor piece correction history

### Search move ordering
- Butterfly history
- Capture history
- Counter move history
- Killer move heuristic
- MVV-LVA

### Time management
- Best move stability

## TODO:
- Checkmate distance pruning
- Continuation history
- Static exchange evaluation
- Tablebases
- Opening book
