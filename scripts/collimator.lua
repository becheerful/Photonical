function update(collimators, dt)
    for _, collimator in ipairs(collimators) do
        local neighboor = get_block_at(collimator.pos[1] + 1, collimator.pos[2])
        print(get_name(neighboor))
    end
end
