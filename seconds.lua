function seconds_hook()
    local time_pos = mp.get_property_number("time-pos")

    -- time_pos can be nil when closing
    if time_pos ~= nil then
      mp.osd_message(string.format("%.2f", time_pos))
    else
      mp.osd_message("N/A")
    end
end

-- mp.register_event("tick", seconds_hook)

