-- Undertone: Elgato Wave:3 Setup Script
-- Creates virtual sink nodes for proper audio routing with Undertone daemon.
--
-- This script:
-- 1. Creates a null sink to keep the Wave:3 source active
-- 2. Creates a playback sink (wave3-sink) for headphone output
-- 3. Links the Wave:3 source to the null sink

log = Log.open_topic("s-elgato_wave3")

-- Object managers
waveSourceOM = ObjectManager({
	Interest({
		type = "node",
		Constraint({ "node.name", "matches", "wave3-source" }),
	}),
})

linkOM = ObjectManager({ Interest({ type = "link" }) })
deviceOM = ObjectManager({ Interest({ type = "device" }) })

-- Globals
waveSinkNode = nil
nullSinkNode = nil
waveLink = nil

-- Create a null sink to keep the Wave:3 source active
-- This prevents the mic from going to sleep when nothing is recording
function create_null_sink()
	local props = {
		["factory.name"] = "support.null-audio-sink",
		["node.name"] = "wave3-null-sink",
		["node.description"] = "Wave:3 Null Sink (internal)",
		["media.class"] = "Audio/Sink",
		["audio.channels"] = "1",
		["audio.position"] = "MONO",
		["monitor.channel-volumes"] = "true",
		["monitor.passthrough"] = "true",
		["node.passive"] = "false",
	}

	log:notice("Creating Wave:3 null sink")
	local node = Node("adapter", props)
	node:activate(Feature.Proxy.BOUND, function(n, err)
		if err then
			log:warning("Failed to create null sink: " .. tostring(err))
			node = nil
		else
			log:notice("Created Wave:3 null sink, id=" .. n.properties["object.id"])
		end
	end)

	return node
end

-- Create a playback sink using the same ALSA device as the Wave:3 source
-- This gives us a proper sink (wave3-sink) for headphone output
function create_wave_sink(source)
	local devInterest = Interest({
		type = "device",
		Constraint({ "object.id", "equals", source.properties["device.id"] }),
	})

	for dev in deviceOM:iterate(devInterest) do
		local props = {
			["device.id"] = source.properties["device.id"],
			["factory.name"] = "api.alsa.pcm.sink",
			["node.name"] = "wave3-sink",
			["node.description"] = "Wave:3 Playback Sink",
			["media.class"] = "Audio/Sink",
			["api.alsa.path"] = source.properties["api.alsa.path"],
			["api.alsa.pcm.card"] = source.properties["api.alsa.pcm.card"],
			["api.alsa.pcm.stream"] = "playback",
			["alsa.resolution_bits"] = "24",
			["audio.channels"] = "2",
			["audio.position"] = "FL,FR",
			["priority.driver"] = "1000",
			["priority.session"] = "1000",
			["node.pause-on-idle"] = "false",
		}

		-- Copy ALSA card properties from the device
		for k, v in pairs(dev.properties) do
			if k:find("^api%.alsa%.card%..*") then
				props[k] = v
			end
		end

		log:notice("Creating Wave:3 playback sink for " .. source.properties["api.alsa.path"])
		waveSinkNode = Node("adapter", props)
		waveSinkNode:activate(Feature.Proxy.BOUND, function(n, err)
			if err then
				log:warning("Failed to create Wave:3 sink: " .. tostring(err))
				waveSinkNode = nil
			else
				log:notice("Created Wave:3 sink, id=" .. n.properties["object.id"])
			end
		end)
	end
end

-- Link Wave:3 source to the null sink to keep the mic active
function create_link(source)
	local portOM = ObjectManager({ Interest({ type = "port" }) })

	local outInterest = Interest({
		type = "port",
		Constraint({ "node.id", "equals", source.properties["object.id"] }),
		Constraint({ "port.direction", "equals", "out" }),
	})

	local inInterest = Interest({
		type = "port",
		Constraint({ "node.id", "equals", nullSinkNode.properties["object.id"] }),
		Constraint({ "port.direction", "equals", "in" }),
	})

	function try_link()
		if waveLink then
			return
		end

		local outPort, inPort
		for p in portOM:iterate(outInterest) do
			outPort = p
		end
		for p in portOM:iterate(inInterest) do
			inPort = p
		end

		if not (outPort and inPort) then
			return
		end

		local args = {
			["link.input.node"] = nullSinkNode.properties["object.id"],
			["link.input.port"] = inPort.properties["object.id"],
			["link.output.node"] = source.properties["object.id"],
			["link.output.port"] = outPort.properties["object.id"],
		}

		log:notice("Linking Wave:3 source -> null sink")
		waveLink = Link("link-factory", args)
		waveLink:activate(Feature.Proxy.BOUND, function(n, err)
			if err then
				log:warning("Failed to link Wave:3 source: " .. tostring(err))
			else
				log:notice("Created link Wave:3 source -> null sink")
			end
		end)
	end

	portOM:connect("object-added", try_link)
	portOM:activate()
end

-- Event handlers
function on_source_added(_, node)
	create_link(node)
end

function on_source_removed()
	if waveSinkNode then
		log:notice("Removing Wave:3 sink")
		waveSinkNode:request_destroy()
		waveSinkNode = nil
	end

	if waveLink then
		log:notice("Removing Wave:3 link")
		waveLink:request_destroy()
		waveLink = nil
	end
end

function on_link_added(_, link)
	if waveLink and link.properties["object.id"] == waveLink.properties["object.id"] then
		for node in waveSourceOM:iterate() do
			create_wave_sink(node)
		end
	end
end

-- Activation
nullSinkNode = create_null_sink()
deviceOM:activate()
linkOM:activate()
linkOM:connect("object-added", on_link_added)
waveSourceOM:connect("object-added", on_source_added)
waveSourceOM:connect("object-removed", on_source_removed)
waveSourceOM:activate()

log:notice("Undertone: Elgato Wave:3 script initialized")
