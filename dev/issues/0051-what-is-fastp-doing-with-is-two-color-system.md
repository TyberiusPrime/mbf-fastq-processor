status: closed
# what is fastp doing with 'is_two_color_system'?

I think it's whether we enable polyG trimming.
line 500 main.cpp
```
    if(!cmd.exist("trim_poly_g") && !cmd.exist("disable_trim_poly_g") && supportEvaluation) {
        bool twoColorSystem = eva.isTwoColorSystem();
```

and it ends up being
    if(starts_with(r->mName, "@NS") || starts_with(r->mName, "@NB") || starts_with(r->mName, "@NDX") || starts_with(r->mName, "@A0")) {



