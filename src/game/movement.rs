 
#![allow(unused)]

type Offset = (isize, isize);



/*
const UP_LEFT: Offset = (-1, -1);
const UP_RIGHT: Offset = (-1, 1);
const DOWN_LEFT: Offset = (1, -1);
const DOWN_RIGHT: Offset = (1, 1);
 */

 
const UP: Offset = (-1, 0);
const DOWN: Offset = (1, 0);
const LEFT: Offset = (0, -1);
const RIGHT: Offset = (0, 1);

enum Movement {
    Walk(Offset),
    Step(Vec<Offset>),
    Jump(Vec<Offset>),
}


#[allow(unused)]
const ROOK_MOVEMENTS: &'static [Movement] = &[
    Movement::Walk(UP),
    Movement::Walk(LEFT),
    Movement::Walk(RIGHT),
    Movement::Walk(DOWN),
];

#[allow(unused)]
const KING_MOVEMENTS: &'static [Movement] = &[
    Movement::Walk(UP),
    Movement::Walk(LEFT),
    Movement::Walk(RIGHT),
    Movement::Walk(DOWN),
];