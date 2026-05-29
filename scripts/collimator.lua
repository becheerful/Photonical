function update(collimators, dt)
    local min = math.min
    for _, collimator in ipairs(collimators) do
        collimator.photonic_plasma = min(collimator.photonic_plasma + 1, collimator.max_photonic_plasma)
    end
end
