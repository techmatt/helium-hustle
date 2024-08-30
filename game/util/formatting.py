
def formatSystemUptime(gameSeconds):
    secondsPerMinute = 60
    secondsPerHour = 3600
    secondsPerDay = 86400
    secondsPerYear = 365.25 * secondsPerDay

    years = gameSeconds / secondsPerYear
    days = gameSeconds / secondsPerDay
    hours = gameSeconds / secondsPerHour
    minutes = gameSeconds / secondsPerMinute

    if years >= 1:
        return f"{years:.2f} years"
    elif days >= 1:
        return f"{days:.2f} days"
    elif hours >= 1:
        return f"{hours:.2f} hours"
    elif minutes >= 1:
        return f"{minutes:.2f} minutes"
    else:
        return f"{gameSeconds:.2f} seconds"
