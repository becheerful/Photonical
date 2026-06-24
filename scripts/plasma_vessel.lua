-- ONLY FOR FFI TESTS

function on_mouse_button_down(vessel, dt)
    print("You've clicked on a plasma vessel!", dt)
end

function on_mouse_button_up(vessel, dt)
    print("You've released the button.", dt)
end
