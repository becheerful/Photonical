function update(miners, dt)
    for _, miner in ipairs(miners) do
        miner.timer = miner.timer + 10 * dt
        if miner.timer >= miner.working_time then
            miner.timer = 0
            miner.stored = miner.stored + 1
        end
    end
end

function on_mouse_button_down(miner, dt)
    print("Stored: " .. miner.stored)
end
