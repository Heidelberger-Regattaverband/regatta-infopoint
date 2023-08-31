sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/core/routing/History",
  "sap/ui/model/json/JSONModel"
], function (Controller, History, JSONModel) {
  "use strict";

  return Controller.extend("de.regatta_hd.infopoint.controller.Base", {

    /**
     * Convenience method for accessing the event bus for this component.
     * @public
     * @returns {sap.ui.core.EventBus} the event bus for this component
     */
    getEventBus: function () {
      return this.getOwnerComponent().getEventBus();
    },

    /**
     * Convenience method for accessing the router.
     * @public
     * @returns {sap.ui.core.routing.Router} the router for this component
     */
    getRouter: function () {
      return this.getOwnerComponent().getRouter();
    },

    /**
     * Convenience method for getting the view model by name.
     * @public
     * @param {string} [sName] the model name
     * @returns {sap.ui.model.Model} the model instance
     */
    getViewModel: function (sName) {
      return this.getView().getModel(sName);
    },

    /**
     * Convenience method for setting the view model.
     * @public
     * @param {sap.ui.model.Model} oModel the model instance
     * @param {string} sName the model name
     * @returns {sap.ui.mvc.View} the view instance
     */
    setViewModel: function (oModel, sName) {
      return this.getView().setModel(oModel, sName);
    },

    /**
     * Getter for the resource bundle.
     * @public
     * @returns {sap.base.i18n.ResourceBundle} the resourceModel of the component
     */
    getResourceBundle: function () {
      return this.getOwnerComponent().getModel("i18n").getResourceBundle();
    },

    navBack: function (sTarget) {
      const sPreviousHash = History.getInstance().getPreviousHash();
      if (sPreviousHash) {
        window.history.go(-1);
      } else {
        this.getRouter().navTo(sTarget, {}, undefined, true /* no history*/);
      }
    },

    displayTarget: function (sTarget) {
      this.getRouter().getTargets().display(sTarget);
    },

    i18n: function (sKey, aArgs) {
      return this.getResourceBundle().getText(sKey, aArgs);
    },

    getRegattaId: function () {
      return this.getOwnerComponent().getRegattaId();
    },

    getJSONModel: async function (sURL, oControl) {
      const oModel = new JSONModel();
      await this.updateJSONModel(oModel, sURL, oControl);
      return oModel;
    },

    updateJSONModel: async function (oModel, sURL, oControl) {
      if (oControl) {
        oControl.setBusy(true);
      }
      await oModel.loadData(sURL);
      if (oControl) {
        oControl.setBusy(false);
      }
    }

  });
});