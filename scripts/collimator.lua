function update(collimators, dt)
    for _, collimator in ipairs(collimators) do
        collimator.light = collimator.light + 1
    end
end
