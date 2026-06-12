function update(vessels, dt)
    local min = math.min
    local max = math.max

    for i, vessel in ipairs(vessels) do
        local imbalance = get_imbalance(vessel.net_id)
        if imbalance >= 0 then
            vessel.stored = min(max(vessel.stored + imbalance, 0), vessel.capacity)
        else
            vessel.stored = max(vessel.stored - imbalance, 0)
        end
        print("PLASMA VESSEL #" .. i, imbalance)
    end
end
