// The cabinet's parameter script: the ordinary API, parts found by name.
fn update(p) {
    let door = child("Door");
    rotate(door, door.min, vec3(0, 0, 1), -p.open);
    show(child("Knob", true), p.open < 45);
}
