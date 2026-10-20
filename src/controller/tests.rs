use super::*;

fn read_all(controller: &mut Controller) -> [u8; 8] {
    std::array::from_fn(|_| controller.read())
}

fn latch(controller: &mut Controller) {
    controller.write(1);
    controller.write(0);
}

#[test]
fn reads_buttons_in_order_a_b_select_start_up_down_left_right() {
    let mut controller = Controller::new();
    controller.set_buttons(button::A | button::START | button::RIGHT);

    latch(&mut controller);
    assert_eq!(read_all(&mut controller), [1, 0, 0, 1, 0, 0, 0, 1]);
}

#[test]
fn each_button_maps_to_its_own_read() {
    let buttons = [
        button::A,
        button::B,
        button::SELECT,
        button::START,
        button::UP,
        button::DOWN,
        button::LEFT,
        button::RIGHT,
    ];
    for (position, &pressed) in buttons.iter().enumerate() {
        let mut controller = Controller::new();
        controller.set_buttons(pressed);

        latch(&mut controller);
        let reads = read_all(&mut controller);
        let expected: [u8; 8] = std::array::from_fn(|i| u8::from(i == position));
        assert_eq!(reads, expected, "button {pressed:#010b}");
    }
}

#[test]
fn reads_after_eight_return_one() {
    let mut controller = Controller::new();

    latch(&mut controller);
    read_all(&mut controller);
    assert_eq!(controller.read(), 1);
    assert_eq!(controller.read(), 1);
}

#[test]
fn strobe_high_always_reads_a() {
    let mut controller = Controller::new();
    controller.set_buttons(button::A | button::B);

    controller.write(1);
    assert_eq!(read_all(&mut controller), [1; 8]);
}

#[test]
fn buttons_are_latched_when_strobe_goes_low() {
    // Presses after the latch don't show up until the next latch.
    let mut controller = Controller::new();

    latch(&mut controller);
    controller.set_buttons(button::A);
    assert_eq!(controller.read(), 0);
    latch(&mut controller);
    assert_eq!(controller.read(), 1);
}

#[test]
fn relatching_restarts_from_a() {
    let mut controller = Controller::new();
    controller.set_buttons(button::A);

    latch(&mut controller);
    controller.read();
    controller.read();
    latch(&mut controller);
    assert_eq!(controller.read(), 1);
}

#[test]
fn only_bit_0_of_write_controls_strobe() {
    let mut controller = Controller::new();
    controller.set_buttons(button::B);

    controller.write(0xFE);
    controller.read();
    assert_eq!(controller.read(), 0, "shift register never latched");
}
