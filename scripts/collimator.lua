-- ONLY FOR FFI TESTS

function update(collimators, dt)
    for _, collimator in ipairs(collimators) do
        print("I'm a collimator!")
    end
end

function on_mouse_button_down(collimator, dt)
    print("Don't touch a collimator!")
end
