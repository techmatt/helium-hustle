
def formatSystemUptime(gameSeconds):
    minutes, seconds = divmod(gameSeconds, 60)
    hours, minutes = divmod(minutes, 60)
    days, hours = divmod(hours, 24)
    years, days = divmod(days, 365.25)
    
    if years >= 1:
        return f"{years:.3f} years"
    elif days >= 1:
        return f"{(days + hours/24):.3f} days"
    elif hours >= 1:
        return f"{(hours + minutes/60):.3f} hours"
    elif minutes >= 1:
        return f"{(minutes + seconds/60):.3f} minutes"
    else:
        return f"{seconds:.3f} seconds"