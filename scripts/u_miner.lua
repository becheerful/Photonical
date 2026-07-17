function update(miners, dt)
    for _, miner in ipairs(miners) do
        if get_block_at(miner.pos[1], miner.pos[2]).str_id ~= miner.source then
            goto continue
        end

        miner.timer = miner.timer + dt
        if miner.timer >= miner.working_time then
            miner.timer = 0
            miner.stored = miner.stored + 1
        end
        ::continue::
    end
end

function on_mouse_button_down(miner, _dt)
    print("Stored: " .. miner.stored)
end
