-- ### Configuration ###

-- Slot modes
local slot_mode_count = 100
local slot_modes = {
    { label = "Normal" },
    { label = "Record", button = "record" },
    { label = "Delete", button = "delete" },
    { label = "Quantize", button = "quantize" },
}

-- Column modes
local column_mode_count = 100
local column_modes = {
    {
        id = "stop",
        label = "Stop clip",
        button = "stop-clip",
        action = "Stop",
        absolute_mode = "Normal",
    },
    {
        id = "solo",
        label = "Solo",
        button = "solo",
        action = "Solo",
        absolute_mode = "ToggleButton",
    },
    {
        id = "record-arm",
        label = "Record arm",
        button = "record-arm",
        action = "Arm",
        absolute_mode = "ToggleButton",
    },
    {
        id = "mute",
        label = "Mute",
        button = "mute",
        action = "Mute",
        absolute_mode = "ToggleButton",
    },
    {
        id = "select",
        label = "Track select",
        button = "track-select",
        action = "Select",
        absolute_mode = "ToggleButton",
    },

}

-- Knob modes
local knob_mode_count = 100
local knob_modes = {
    { label = "Volume", button = "volume" },
    { label = "Pan", button = "pan" },
    { label = "Sends", button = "sends" },
    { label = "Device", button = "device" },
}

-- Number of columns and rows
-- TODO-medium Would be good to take this dynamically from the controller preset as a compartment variable.
--- However, at the moment it's not relevant. We just take a reasonable maximum.
local column_count = 8
local row_count = 8

-- ### Content ###

local mappings = {
    {
        name = "Stop all clips",
        source = {
            kind = "Virtual",
            id = "stop-all-clips",
            character = "Button",
        },
        target = {
            kind = "ClipMatrixAction",
            action = "Stop",
        },
    },
    {
        name = "Play arrangement",
        source = {
            kind = "Virtual",
            id = "play",
            character = "Button",
        },
        glue = {
            absolute_mode = "ToggleButton",
        },
        target = {
            kind = "TransportAction",
            action = "PlayPause",
        },
    },
    {
        name = "Scroll up",
        feedback_enabled = false,
        source = {
            kind = "Virtual",
            id = "cursor-up",
            character = "Button",
        },
        glue = {
            absolute_mode = "IncrementalButton",
            reverse = true,
        },
        target = {
            kind = "FxParameterValue",
            parameter = {
                address = "ById",
                index = 1,
            },
        },
    },
    {
        name = "Scroll down",
        feedback_enabled = false,
        source = {
            kind = "Virtual",
            id = "cursor-down",
            character = "Button",
        },
        glue = {
            absolute_mode = "IncrementalButton",
        },
        target = {
            kind = "FxParameterValue",
            parameter = {
                address = "ById",
                index = 1,
            },
        },
    },
    {
        name = "Scroll left",
        feedback_enabled = false,
        source = {
            kind = "Virtual",
            id = "cursor-left",
            character = "Button",
        },
        glue = {
            absolute_mode = "IncrementalButton",
            reverse = true,
        },
        target = {
            kind = "FxParameterValue",
            parameter = {
                address = "ById",
                index = 0,
            },
        },
    },
    {
        name = "Scroll right",
        feedback_enabled = false,
        source = {
            kind = "Virtual",
            id = "cursor-right",
            character = "Button",
        },
        glue = {
            absolute_mode = "IncrementalButton",
        },
        target = {
            kind = "FxParameterValue",
            parameter = {
                address = "ById",
                index = 0,
            },
        },
    },
}

-- Slot modes
local slot_mode_labels = {}
for i, mode in ipairs(slot_modes) do
    table.insert(slot_mode_labels, mode.label)
    if mode.button then
        local target_value = (i - 1) / slot_mode_count
        local m = {
            group = "slot-modes",
            name = mode.label,
            source = {
                kind = "Virtual",
                id = mode.button,
                character = "Button",
            },
            glue = {
                absolute_mode = "ToggleButton",
                target_interval = { target_value, target_value },
                out_of_range_behavior = "Min",
            },
            target = {
                kind = "FxParameterValue",
                parameter = {
                    address = "ById",
                    index = 3,
                },
            },
        }
        table.insert(mappings, m)
    end
end

-- Column modes
local column_mode_labels = {}
for i, mode in ipairs(column_modes) do
    table.insert(column_mode_labels, mode.label)
    local target_value = (i - 1) / column_mode_count
    local m = {
        group = "column-modes",
        name = mode.label,
        source = {
            kind = "Virtual",
            id = mode.button,
            character = "Button",
        },
        glue = {
            target_interval = { target_value, target_value },
            out_of_range_behavior = "Min",
        },
        target = {
            kind = "FxParameterValue",
            parameter = {
                address = "ById",
                index = 4,
            },
        },
    }
    table.insert(mappings, m)
end

-- Knob modes
local knob_mode_labels = {}
for i, mode in ipairs(knob_modes) do
    table.insert(knob_mode_labels, mode.label)
    local target_value = (i - 1) / knob_mode_count
    local m = {
        group = "knob-modes",
        name = mode.label,
        source = {
            kind = "Virtual",
            id = mode.button,
            character = "Button",
        },
        glue = {
            target_interval = { target_value, target_value },
            out_of_range_behavior = "Min",
        },
        target = {
            kind = "FxParameterValue",
            parameter = {
                address = "ById",
                index = 5,
            },
        },
    }
    table.insert(mappings, m)
end

-- Parameters
local parameters = {
    {
        index = 0,
        name = "Column offset",
        value_count = 10000,
    },
    {
        index = 1,
        name = "Row offset",
        value_count = 10000,
    },
    {
        index = 2,
        name = "Shift modifier",
    },
    {
        index = 3,
        name = "Slot mode",
        value_count = slot_mode_count,
        value_labels = slot_mode_labels
    },
    {
        index = 4,
        name = "Column mode",
        value_count = column_mode_count,
        value_labels = column_mode_labels
    },
    {
        index = 5,
        name = "Knob mode",
        value_count = knob_mode_count,
        value_labels = knob_mode_labels
    },
}

local groups = {
    {
        id = "slot-modes",
        name = "Slot modes",
    },
    {
        id = "column-modes",
        name = "Column modes",
    },
    {
        id = "knob-modes",
        name = "Knob modes",
    },
    {
        id = "slot-feedback",
        name = "Slot feedback",
    },
    {
        id = "slot-play",
        name = "Slot play",
        activation_condition = {
            kind = "Bank",
            parameter = 3,
            bank_index = 0,
        },
    },
    {
        id = "slot-record",
        name = "Slot record",
        activation_condition = {
            kind = "Bank",
            parameter = 3,
            bank_index = 1,
        },
    },
    {
        id = "slot-clear",
        name = "Slot clear",
        activation_condition = {
            kind = "Bank",
            parameter = 3,
            bank_index = 2,
        },
    },
    {
        id = "slot-quantize",
        name = "Slot quantize",
        activation_condition = {
            kind = "Bank",
            parameter = 3,
            bank_index = 3,
        },
    },
    {
        id = "column-stop",
        name = "Column stop",
        activation_condition = {
            kind = "Bank",
            parameter = 4,
            bank_index = 0,
        },
    },
    {
        id = "column-solo",
        name = "Column solo",
        activation_condition = {
            kind = "Bank",
            parameter = 4,
            bank_index = 1,
        },
    },
    {
        id = "column-record-arm",
        name = "Column record arm",
        activation_condition = {
            kind = "Bank",
            parameter = 4,
            bank_index = 2,
        },
    },
    {
        id = "column-mute",
        name = "Column mute",
        activation_condition = {
            kind = "Bank",
            parameter = 4,
            bank_index = 3,
        },
    },
    {
        id = "column-select",
        name = "Column select",
        activation_condition = {
            kind = "Bank",
            parameter = 4,
            bank_index = 4,
        },
    },
    {
        id = "knob-volume",
        name = "Knob volume",
        activation_condition = {
            kind = "Bank",
            parameter = 5,
            bank_index = 0,
        },
    },
    {
        id = "knob-pan",
        name = "Knob pan",
        activation_condition = {
            kind = "Bank",
            parameter = 5,
            bank_index = 1,
        },
    },
    {
        id = "knob-sends",
        name = "Knob sends",
        activation_condition = {
            kind = "Bank",
            parameter = 5,
            bank_index = 2,
        },
    },
    {
        id = "knob-device",
        name = "Knob device",
        activation_condition = {
            kind = "Bank",
            parameter = 5,
            bank_index = 3,
        },
    },
}

-- For each column
for col = 0, column_count - 1 do
    local human_col = col + 1
    local prefix = "col" .. human_col .. "/"
    local column_expression = "p[0] + " .. col
    -- Buttons
    for _, button in ipairs(column_modes) do
        local mapping = {
            name = "Column " .. human_col .. " " .. button.id,
            group = "column-" .. button.id,
            source = {
                kind = "Virtual",
                character = "Button",
                id = prefix .. "stop",
            },
            glue = {
                absolute_mode = button.absolute_mode,
            },
            target = {
                kind = "ClipColumnAction",
                column = {
                    address = "Dynamic",
                    expression = column_expression,
                },
                action = button.action,
            },
        }
        table.insert(mappings, mapping)
    end
    -- Knob
    for _, button in ipairs(knob_modes) do
        --local mapping = {
        --    name = "Column " .. human_col .. " " .. button.id,
        --    group = "column-" .. button.id,
        --    source = {
        --        kind = "Virtual",
        --        character = "Button",
        --        id = prefix .. "stop",
        --    },
        --    glue = {
        --        absolute_mode = button.absolute_mode,
        --    },
        --    target = {
        --        kind = "ClipColumnAction",
        --        column = {
        --            address = "Dynamic",
        --            expression = column_expression,
        --        },
        --        action = button.action,
        --    },
        --}
        --table.insert(mappings, mapping)
    end
end

-- For each slot
for col = 0, column_count - 1 do
    local human_col = col + 1
    for row = 0, row_count - 1 do
        local human_row = row + 1
        local prefix = "col" .. human_col .. "/row" .. human_row .. "/"
        local slot_column_expression = "p[0] + " .. col
        local slot_row_expression = "p[1] + " .. row
        local slot_play = {
            id = prefix .. "slot-play",
            name = "Slot " .. human_col .. "/" .. human_row .. " play",
            group = "slot-play",
            feedback_enabled = false,
            source = {
                kind = "Virtual",
                character = "Button",
                id = prefix .. "pad",
            },
            glue = {
                absolute_mode = "ToggleButton",
            },
            target = {
                kind = "ClipTransportAction",
                slot = {
                    address = "Dynamic",
                    column_expression = slot_column_expression,
                    row_expression = slot_row_expression
                },
                action = "RecordPlayStop",
                record_only_if_track_armed = true,
            },
        }
        local slot_play_feedback = {
            id = prefix .. "slot-play-feedback",
            name = "Slot " .. human_col .. "/" .. human_row .. " play feedback",
            group = "slot-feedback",
            control_enabled = false,
            source = {
                kind = "Virtual",
                character = "Button",
                id = prefix .. "pad",
            },
            glue = {
                feedback = {
                    kind = "Text",
                    text_expression = "{{ target.slot_state.id }}",
                },
            },
            target = {
                kind = "ClipTransportAction",
                slot = {
                    address = "Dynamic",
                    column_expression = slot_column_expression,
                    row_expression = slot_row_expression
                },
                action = "PlayStop",
            },
        }
        local slot_record = {
            id = prefix .. "slot-record",
            name = "Slot " .. human_col .. "/" .. human_row .. " record",
            group = "slot-record",
            feedback_enabled = false,
            source = {
                kind = "Virtual",
                character = "Button",
                id = prefix .. "pad",
            },
            glue = {
                absolute_mode = "ToggleButton",
            },
            target = {
                kind = "ClipTransportAction",
                slot = {
                    address = "Dynamic",
                    column_expression = slot_column_expression,
                    row_expression = slot_row_expression
                },
                action = "RecordStop",
            },
        }
        local slot_clear = {
            id = prefix .. "slot-clear",
            name = "Slot " .. human_col .. "/" .. human_row .. " clear",
            group = "slot-clear",
            feedback_enabled = false,
            source = {
                kind = "Virtual",
                character = "Button",
                id = prefix .. "pad",
            },
            target = {
                kind = "ClipManagement",
                slot = {
                    address = "Dynamic",
                    column_expression = slot_column_expression,
                    row_expression = slot_row_expression
                },
                action = {
                    kind = "ClearSlot",
                },
            },
        }
        local slot_quantize = {
            id = prefix .. "slot-quantize",
            name = "Slot " .. human_col .. "/" .. human_row .. " quantize",
            group = "slot-quantize",
            feedback_enabled = false,
            source = {
                kind = "Virtual",
                character = "Button",
                id = prefix .. "pad",
            },
            glue = {
                absolute_mode = "ToggleButton",
            },
            target = {
                kind = "ClipManagement",
                slot = {
                    address = "Dynamic",
                    column_expression = slot_column_expression,
                    row_expression = slot_row_expression
                },
                action = {
                    kind = "EditClip",
                },
            },
        }
        table.insert(mappings, slot_play)
        table.insert(mappings, slot_play_feedback)
        table.insert(mappings, slot_record)
        table.insert(mappings, slot_clear)
        table.insert(mappings, slot_quantize)
    end
end

return {
    kind = "MainCompartment",
    value = {
        parameters = parameters,
        groups = groups,
        mappings = mappings,
    },
}