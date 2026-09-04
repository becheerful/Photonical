function update(world, ecs, miners, dt)
    for _, miner in ipairs(miners) do
        if get_str_id(
            get_raw_id(ecs, get_tile_uid_at(world, miner.pos[1], miner.pos[2]))
        ) ~= miner.source then
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

function on_mouse_button_down(_world, _ecs, miner, _dt)
    print("Stored: " .. miner.stored)
end
