sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/model/json/JSONModel",
  "../model/Formatter"
], function (Controller, JSONModel, Formatter) {
  "use strict";
  return Controller.extend("de.regatta_hd.infopoint.controller.Heats", {

    formatter: Formatter,

    onInit: function () {
      const oComponent = this.getOwnerComponent();

      this.getView().addStyleClass(oComponent.getContentDensityClass());

      oComponent.getRouter().getRoute("heats")
        .attachPatternMatched(this._loadHeatsModel, this);
    },

    onNavBack: function () {
      const oRouter = this.getOwnerComponent().getRouter();
      oRouter.navTo("startpage", {}, true);
    },

    _loadHeatsModel: function () {
      const oRegatta = this.getOwnerComponent().getModel("regatta").getData();
      const oHeatsModel = new JSONModel();
      oHeatsModel.loadData("/api/regattas/" + oRegatta.id + "/heats");
      this.getOwnerComponent().setModel(oHeatsModel, "heats");
    }

  });
});