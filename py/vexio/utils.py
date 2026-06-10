from datetime import datetime


def vex_time(dt: datetime) -> str:
    """Convert a Python `datetime` into a VEX formatted datetime string"""
    return dt.strftime("%Yy%jd%Hh%Mm%Ss")
