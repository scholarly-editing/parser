from py_parser import Passage

ex = Passage("ترفعه مروءته من المنزلة الوضيعة إلى المنزلة الرفيعة")


print(ex.is_valid())


print(ex.locate_fragment("المنزلة الوضيعة"))

print(ex.locate_fragment("المنزلة الرفيعة"))


print(ex.locate_fragment("ترفعه مروءته"))
