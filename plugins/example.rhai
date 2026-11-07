fn on_enable() {
    print("Example plugin enabled");
}

fn on_tick(tick) {
    if tick % 1200 == 0 {
        print("Lithum has been running for about one minute");
    }
}

fn on_console_command(command) {
    if command == "status" {
        print("Example plugin saw the status command");
    }
}
