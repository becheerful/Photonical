function update(collimators, dt)
    for _, collimator in ipairs(collimators) do
        print(get_name(collimator.block_id))
    end
end
