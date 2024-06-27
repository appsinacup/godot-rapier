extends Node2D


func _ready() -> void:
	var space := get_viewport().find_world_2d().direct_space_state as RapierDirectSpaceState2D
	var space_json := FileAccess.open("user://space.json", FileAccess.WRITE)
	var shapes_json := FileAccess.open("user://shapes.json", FileAccess.WRITE)
	#space_json.store_string(space.export_json())
	#shapes_json.store_string(RapierPhysicsServer2D.shapes_export_json())
