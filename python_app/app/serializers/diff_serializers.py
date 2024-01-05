def allDiffsResponseEntity(diff):
    temp_diff = diff.copy()
    del temp_diff["created_at"]
    if "updated_at" in temp_diff:
        del temp_diff["updated_at"]
    return temp_diff
